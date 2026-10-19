//! Ordinary child positions and exact, tree-ordered Slot participants.
//! Relation deltas own topology here; the separately committed parent scalar
//! catches up before the next source controller. Shadow/Template host edges
//! never enter this private order.
use super::super::*;
use super::NodeFields;
pub(in crate::store) use super::ordered_members::ChildPosition;
use super::ordered_members::{
    Edit as Editing, Map, MapCustody, Members, MembersCustody, Membership, Mode, OrderView,
    Qualified, RootHeader,
};

#[cfg(test)]
fn discovery() {
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::SlotDiscoveryStep);
    }
}

fn step(work: &mut dyn FnMut() -> Result<(), DomError>) -> Result<(), DomError> {
    work()?;
    #[cfg(test)]
    discovery();
    Ok(())
}

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

/// These canonical values carry no accounting authority.
#[derive(Clone, Debug, Default)]
pub(in crate::store) struct SlotIndex {
    positions: Map<ChildPosition>,
    members: Members,
    roots: Map<Root>,
    pub(in crate::store) named: super::slot_names::Index,
}
#[derive(Clone, Default)]
pub(in crate::store) struct SlotCustody {
    positions: MapCustody<ChildPosition>,
    members: MembersCustody,
    roots: MapCustody<Root>,
    pub(in crate::store) named: super::slot_names::Custody,
}
#[derive(Clone, Default)]
struct Version {
    index: SlotIndex,
    dirty_positions: Map<Key>,
    dirty_members: Map<Key>,
    custody: SlotCustody,
    dirty_positions_custody: MapCustody<Key>,
    dirty_members_custody: MapCustody<Key>,
}
pub(in crate::store) struct Slots {
    base: SlotIndex,
    current: Version,
    base_custody: SlotCustody,
}
pub(in crate::store) struct Finished {
    pub value: SlotIndex,
    pub custody: SlotCustody,
}
pub(in crate::store) struct SlotPlan {
    next: Version,
    retain_undo: bool,
}
pub(in crate::store) struct SlotUndo {
    before: Version,
}
impl SlotUndo {
    pub fn restore(self, draft: &mut Draft) {
        draft.slot_index.current = self.before;
    }
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        0
    }
}
struct Reconcile(bank::CustodyRegistry);
impl Drop for Reconcile {
    fn drop(&mut self) {
        self.0.reconcile_releases();
    }
}
impl SlotIndex {
    pub fn position(&self, key: Key) -> Option<ChildPosition> {
        self.positions.get(key).copied()
    }
    fn root(
        &self,
        key: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Key, DomError> {
        OrderView::positions(self).root(key, Mode::Ordinary, work)
    }
    fn descendant(
        &self,
        key: Key,
        ancestor: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<bool, DomError> {
        OrderView::positions(self).descendant(ancestor, key, Mode::Ordinary, work)
    }
    pub fn first(
        &self,
        root: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let mut observed = || step(work);
        observed()?;
        self.members.first(
            self.roots
                .get(root)
                .map_or_else(RootHeader::default, |r| r.member),
            &mut observed,
        )
    }
    pub fn next(
        &self,
        root: Key,
        key: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.next_unobserved(root, key, &mut || step(work))
    }
    fn next_unobserved(
        &self,
        root: Key,
        key: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        work()?;
        self.members.after(
            self.roots
                .get(root)
                .map_or_else(RootHeader::default, |r| r.member),
            key,
            OrderView::positions(self),
            Mode::Ordinary,
            work,
        )
    }
    pub(super) fn lower_bound(
        &self,
        root: Key,
        key: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.lower_bound_unobserved(root, key, &mut || step(work))
    }
    fn lower_bound_unobserved(
        &self,
        root: Key,
        key: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        work()?;
        self.members.lower_bound(
            self.roots
                .get(root)
                .map_or_else(RootHeader::default, |r| r.member),
            key,
            OrderView::positions(self),
            Mode::Ordinary,
            work,
        )
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        work: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<SlotCustody, DomError> {
        fn adopt<T: Send + Sync + 'static>(
            map: &Map<T>,
            registry: &bank::CustodyRegistry,
            admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
            work: &mut impl FnMut() -> Result<(), DomError>,
        ) -> Result<MapCustody<T>, DomError> {
            let existing = map.existing.adopt_controlled(
                registry,
                |_| {
                    step(work)?;
                    Ok(custody::Usage::new(size_of::<T>(), 0))
                },
                &mut *admit,
            )?;
            let created = map.created.adopt_controlled(
                registry,
                |_| {
                    step(work)?;
                    Ok(custody::Usage::new(size_of::<T>(), 0))
                },
                admit,
            )?;
            Ok(MapCustody { existing, created })
        }
        let _cleanup = Reconcile(registry.clone());
        Ok(SlotCustody {
            positions: adopt(&self.positions, registry, admit, work)?,
            members: self
                .members
                .adopt_custody(registry, admit, &mut || step(work))?,
            roots: adopt(&self.roots, registry, admit, work)?,
            named: self.named.adopt_custody(registry, admit, work)?,
        })
    }
}

#[cfg(test)]
pub(in crate::store) struct SlotMemberInspection {
    pub key: Key,
    pub root: Key,
    pub left: Option<Key>,
    pub right: Option<Key>,
    pub order_parent: Option<Key>,
    pub height: u8,
}
#[cfg(test)]
impl SlotIndex {
    pub fn inspect_positions(&self) -> Vec<ChildPosition> {
        self.positions.values().copied().collect()
    }
    pub fn inspect_members(&self) -> Vec<SlotMemberInspection> {
        self.roots
            .values()
            .flat_map(|group| {
                self.members
                    .inspect_tree(group.member)
                    .into_iter()
                    .map(move |m| SlotMemberInspection {
                        key: m.key,
                        root: group.key,
                        left: m.left,
                        right: m.right,
                        order_parent: m.parent,
                        height: m.height,
                    })
            })
            .collect()
    }
    pub fn inspect_roots(&self) -> Vec<(Key, Key)> {
        self.roots
            .values()
            .filter_map(|r| self.members.inspect_root(r.member).map(|key| (r.key, key)))
            .collect()
    }
    pub fn inspect_summaries(&self) -> Vec<(bank::Totals, bank::IndexCharge)> {
        let mut summaries = vec![
            (
                self.positions.existing.totals(),
                self.positions.existing.index_charge(),
            ),
            (
                self.positions.created.totals(),
                self.positions.created.index_charge(),
            ),
            self.members.inspect_summary(),
            (
                self.roots.existing.totals(),
                self.roots.existing.index_charge(),
            ),
            (
                self.roots.created.totals(),
                self.roots.created.index_charge(),
            ),
        ];
        summaries.extend(self.named.inspect_summaries());
        summaries
    }
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        fn graph<T: Qualified>(map: &Map<T>) -> Result<bank::PhysicalGraph, DomError> {
            for (index, value) in map.existing.iter().chain(map.created.iter()) {
                let expected = match value.key() {
                    Key::Existing(id) => id.slot,
                    Key::Created(index) => index,
                };
                assert_eq!(
                    index, expected,
                    "actual qualified Slot metadata storage index"
                );
            }
            let describe = |_: &T| {
                Ok(bank::Totals {
                    bytes: size_of::<T>(),
                    units: 0,
                    occurrences: 1,
                })
            };
            let mut graph = map.existing.inspect_graph(describe)?;
            graph.merge(&map.created.inspect_graph(describe)?)?;
            Ok(graph)
        }
        let mut output = graph(&self.positions)?;
        output.merge(&self.members.inspect_graph()?)?;
        output.merge(&graph(&self.roots)?)?;
        output.merge(&self.named.inspect_graph()?)?;
        Ok(output)
    }
}

impl Version {
    fn remove_member(&mut self, key: Key, edit: &mut Editing<'_>) -> Result<(), DomError> {
        let root = self.index.root(key, edit.step)?;
        let Some(header) = self.index.roots.get(root).copied() else {
            return Ok(());
        };
        let changed = self.index.members.change(
            &self.custody.members,
            header.member,
            Membership::Remove(key),
            OrderView::positions(&self.index),
            Mode::Ordinary,
            Editing {
                registry: edit.registry,
                admit: edit.admit,
                step: edit.step,
            },
        )?;
        self.index.members = changed.members;
        self.custody.members = changed.custody;
        self.index.roots.change(
            &mut self.custody.roots,
            root,
            (!changed.root.is_empty()).then_some(Root {
                key: root,
                member: changed.root,
            }),
            edit,
        )?;
        Ok(())
    }
    fn insert_member(&mut self, key: Key, edit: &mut Editing<'_>) -> Result<(), DomError> {
        let root = self.index.root(key, edit.step)?;
        let header = self
            .index
            .roots
            .get(root)
            .map_or_else(RootHeader::default, |r| r.member);
        let changed = self.index.members.change(
            &self.custody.members,
            header,
            Membership::Insert(key),
            OrderView::positions(&self.index),
            Mode::Ordinary,
            Editing {
                registry: edit.registry,
                admit: edit.admit,
                step: edit.step,
            },
        )?;
        self.index.members = changed.members;
        self.custody.members = changed.custody;
        self.index.roots.change(
            &mut self.custody.roots,
            root,
            Some(Root {
                key: root,
                member: changed.root,
            }),
            edit,
        )?;
        Ok(())
    }
    fn position(
        &mut self,
        key: Key,
        value: Option<ChildPosition>,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        if self
            .index
            .positions
            .change(&mut self.custody.positions, key, value, edit)?
        {
            self.dirty_positions
                .change(&mut self.dirty_positions_custody, key, Some(key), edit)?;
        }
        Ok(())
    }
    fn dirty_member(&mut self, key: Key, edit: &mut Editing<'_>) -> Result<(), DomError> {
        self.dirty_members
            .change(&mut self.dirty_members_custody, key, Some(key), edit)?;
        Ok(())
    }
    fn subtree(
        &mut self,
        source: &SlotIndex,
        node: Key,
        insert: bool,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        let root = source.root(node, edit.step)?;
        let mut current = source.lower_bound_unobserved(root, node, edit.step)?;
        while let Some(key) = current {
            if !source.descendant(key, node, edit.step)? {
                break;
            }
            current = source.next_unobserved(root, key, edit.step)?;
            if insert {
                self.insert_member(key, edit)?;
            } else {
                self.remove_member(key, edit)?;
                self.dirty_member(key, edit)?;
            }
        }
        Ok(())
    }
}

impl Slots {
    pub(super) fn index(&self) -> &SlotIndex {
        &self.current.index
    }
    pub fn order<'a>(&'a self, draft: &'a Draft) -> super::ordered_members::OrderView<'a> {
        super::ordered_members::OrderView::draft(draft, &self.current.index)
    }
    pub fn capture(base: &SlotIndex, custody: &SlotCustody) -> Self {
        Self {
            base: base.clone(),
            base_custody: custody.clone(),
            current: Version {
                index: base.clone(),
                custody: custody.clone(),
                ..Default::default()
            },
        }
    }
    pub fn position(&self, key: Key) -> Option<ChildPosition> {
        self.current.index.position(key)
    }
    pub fn first(
        &self,
        root: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.current.index.first(root, work)
    }
    pub fn next(
        &self,
        root: Key,
        key: Key,
        work: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.current.index.next(root, key, work)
    }
    pub fn mark_published(&mut self, base: &SlotIndex, custody: &SlotCustody) {
        self.base = base.clone();
        self.base_custody = custody.clone();
        self.current.index.named = base.named.clone();
        self.current.custody.named = custody.named.clone();
        self.current.dirty_positions = Default::default();
        self.current.dirty_members = Default::default();
        self.current.dirty_positions_custody = Default::default();
        self.current.dirty_members_custody = Default::default();
    }
}

impl SlotPlan {
    pub fn order<'a>(&'a self, draft: &'a Draft) -> super::ordered_members::OrderView<'a> {
        super::ordered_members::OrderView::draft(draft, &self.next.index)
    }
}

impl Draft {
    fn slot_plan(
        &self,
        mut change: impl FnMut(&mut Version, &mut Editing<'_>) -> Result<(), DomError>,
        cancel: &AtomicBool,
    ) -> Result<SlotPlan, DomError> {
        let retain_undo = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut next = self.slot_index.current.clone();
        change(
            &mut next,
            &mut Editing {
                registry: self.custody_registry(),
                admit: &mut |usage| self.admit_custody_usage(usage, cancel),
                step: &mut || step(&mut || self.source_step(cancel)),
            },
        )?;
        Ok(SlotPlan { next, retain_undo })
    }
    pub(in crate::store) fn commit_slot_plan(&mut self, plan: SlotPlan) {
        let before = std::mem::replace(&mut self.slot_index.current, plan.next);
        if plan.retain_undo {
            self.journal
                .push_prepared(super::Undo::Slots(SlotUndo { before }));
        }
    }
    pub(in crate::store) fn prepare_slot_children(
        &self,
        parent: Key,
        index: usize,
        removed: usize,
        added: &[Key],
        cancel: &AtomicBool,
    ) -> Result<Option<SlotPlan>, DomError> {
        if removed == 0 && added.is_empty() {
            return Ok(None);
        }
        let parent = self.live_key(parent);
        let old = self.node(parent)?.children;
        let end = index
            .checked_add(removed)
            .filter(|end| *end <= old.len())
            .ok_or(DomError::IndexSize)?;
        let source = &self.slot_index.current.index;
        self.slot_plan(
            |next, edit| {
                for child in (index..end)
                    .map(|i| old.get(i).expect("selected child"))
                    .chain(added.iter().copied())
                {
                    next.subtree(source, self.live_key(child), false, edit)?;
                }
                for child in (index..end).map(|i| old.get(i).expect("removed child")) {
                    next.position(self.live_key(child), None, edit)?;
                }
                for ordinal in end..old.len() {
                    let key = self.live_key(old.get(ordinal).expect("shifted child"));
                    let ordinal = ordinal - removed + added.len();
                    next.position(
                        key,
                        Some(ChildPosition {
                            key,
                            parent,
                            ordinal,
                        }),
                        edit,
                    )?;
                }
                for (offset, &child) in added.iter().enumerate() {
                    let key = self.live_key(child);
                    let ordinal = index
                        .checked_add(offset)
                        .ok_or(DomError::RepresentationExhausted)?;
                    next.position(
                        key,
                        Some(ChildPosition {
                            key,
                            parent,
                            ordinal,
                        }),
                        edit,
                    )?;
                }
                for child in (index..end)
                    .map(|i| old.get(i).expect("selected child"))
                    .chain(added.iter().copied())
                {
                    next.subtree(source, self.live_key(child), true, edit)?;
                }
                Ok(())
            },
            cancel,
        )
        .map(Some)
    }
    pub(in crate::store) fn prepare_slot_children_replace(
        &self,
        parent: Key,
        added: &[Key],
        cancel: &AtomicBool,
    ) -> Result<Option<SlotPlan>, DomError> {
        let parent = self.live_key(parent);
        let old = self.node(parent)?.children;
        if old
            .iter()
            .eq(added.iter().copied().map(|key| self.live_key(key)))
        {
            return Ok(None);
        }
        let source = &self.slot_index.current.index;
        self.slot_plan(
            |next, edit| {
                for child in old.iter().chain(added.iter().copied().filter(|key| {
                    source
                        .position(self.live_key(*key))
                        .is_none_or(|p| p.parent != parent)
                })) {
                    next.subtree(source, self.live_key(child), false, edit)?;
                }
                for child in old.iter() {
                    next.position(self.live_key(child), None, edit)?;
                }
                for (ordinal, &child) in added.iter().enumerate() {
                    let key = self.live_key(child);
                    next.position(
                        key,
                        Some(ChildPosition {
                            key,
                            parent,
                            ordinal,
                        }),
                        edit,
                    )?;
                }
                for child in old.iter().chain(added.iter().copied().filter(|key| {
                    source
                        .position(self.live_key(*key))
                        .is_none_or(|p| p.parent != parent)
                })) {
                    next.subtree(source, self.live_key(child), true, edit)?;
                }
                Ok(())
            },
            cancel,
        )
        .map(Some)
    }
    pub(in crate::store) fn prepare_slot_created(
        &self,
        key: Key,
        fields: &NodeFields,
        cancel: &AtomicBool,
    ) -> Result<Option<SlotPlan>, DomError> {
        if !crate::slot::is_slot(&fields.kind) {
            return Ok(None);
        }
        self.slot_plan(
            |next, edit| {
                next.insert_member(key, edit)?;
                next.dirty_member(key, edit)
            },
            cancel,
        )
        .map(Some)
    }
    pub(in crate::store) fn prepare_slot_fields(
        &self,
        key: Key,
        fields: &NodeFields,
        cancel: &AtomicBool,
    ) -> Result<Option<SlotPlan>, DomError> {
        let key = self.live_key(key);
        let was = crate::slot::is_slot(self.node(key)?.kind);
        let is = crate::slot::is_slot(&fields.kind);
        if was == is {
            return Ok(None);
        }
        self.slot_plan(
            |next, edit| {
                if is {
                    next.insert_member(key, edit)?;
                } else {
                    next.remove_member(key, edit)?;
                }
                next.dirty_member(key, edit)
            },
            cancel,
        )
        .map(Some)
    }
    pub(in crate::store) fn prepare_slot_retired(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<SlotPlan>, DomError> {
        let key = self.live_key(key);
        if self.slot_index.position(key).is_none() && !crate::slot::is_slot(self.node(key)?.kind) {
            return Ok(None);
        }
        self.slot_plan(
            |next, edit| {
                if crate::slot::is_slot(self.node(key)?.kind) {
                    next.remove_member(key, edit)?;
                    next.dirty_member(key, edit)?;
                }
                next.position(key, None, edit)
            },
            cancel,
        )
        .map(Some)
    }
    pub(in crate::store) fn prepare_slot_visibility(
        &self,
        key: Key,
        was_private: bool,
        is_private: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<SlotPlan>, DomError> {
        let key = self.live_key(key);
        if was_private == is_private {
            return Ok(None);
        }
        self.slot_plan(
            |next, edit| {
                next.dirty_positions.change(
                    &mut next.dirty_positions_custody,
                    key,
                    Some(key),
                    edit,
                )?;
                if crate::slot::is_slot(self.node(key)?.kind) {
                    next.dirty_member(key, edit)?;
                }
                Ok(())
            },
            cancel,
        )
        .map(Some)
    }
    pub(in crate::store) fn finish_slot_index(
        &self,
        mapper: &impl Fn(Key) -> Option<NodeId>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut output = Version {
            index: self.slot_index.base.clone(),
            custody: self.slot_index.base_custody.clone(),
            ..Default::default()
        };
        let mut edit = Editing {
            registry: self.custody_registry(),
            admit: &mut |usage| self.admit_custody_usage(usage, cancel),
            step: &mut || step(&mut || self.source_step(cancel)),
        };
        // Remove all old affected members before the final comparator changes.
        for key in self.slot_index.current.dirty_members.values().copied() {
            if let Some(id) = mapper(key) {
                output.remove_member(Key::Existing(id), &mut edit)?;
            }
        }
        for key in self.slot_index.current.dirty_positions.values().copied() {
            (edit.step)()?;
            if let Some(id) = mapper(key) {
                output.index.positions.change(
                    &mut output.custody.positions,
                    Key::Existing(id),
                    None,
                    &mut edit,
                )?;
                if let Some(position) = self.slot_index.position(self.live_key(key))
                    && mapper(position.parent).is_some()
                {
                    output.dirty_positions.change(
                        &mut output.dirty_positions_custody,
                        position.parent,
                        Some(position.parent),
                        &mut edit,
                    )?;
                }
            }
        }
        // Canonical Children omit private nodes. Compute their filtered
        // ordinals once per changed parent field, never once per child prefix.
        for parent in output.dirty_positions.values().copied() {
            let Some(parent_id) = mapper(parent) else {
                continue;
            };
            let mut ordinal = 0usize;
            for child in self.node(parent)?.children.iter() {
                (edit.step)()?;
                if let Some(id) = mapper(child) {
                    let key = Key::Existing(id);
                    output.index.positions.change(
                        &mut output.custody.positions,
                        key,
                        Some(ChildPosition {
                            key,
                            parent: Key::Existing(parent_id),
                            ordinal,
                        }),
                        &mut edit,
                    )?;
                    ordinal = ordinal
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
        }
        for key in self.slot_index.current.dirty_members.values().copied() {
            (edit.step)()?;
            if self
                .node(self.live_key(key))
                .is_ok_and(|node| crate::slot::is_slot(node.kind))
                && let Some(id) = mapper(key)
            {
                output.insert_member(Key::Existing(id), &mut edit)?;
            }
        }
        Ok(Finished {
            value: output.index,
            custody: output.custody,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn assert_unchanged_index(actual: &SlotIndex, before: &SlotIndex) {
        assert_eq!(actual.positions, before.positions);
        assert_eq!(actual.members, before.members);
        assert_eq!(actual.roots, before.roots);
        let actual_graph = actual.inspect_graph().unwrap();
        let before_graph = before.inspect_graph().unwrap();
        assert_eq!(
            actual_graph.pages().collect::<Vec<_>>(),
            before_graph.pages().collect::<Vec<_>>()
        );
        assert_eq!(
            actual_graph.records().collect::<Vec<_>>(),
            before_graph.records().collect::<Vec<_>>()
        );
        let groups = |index: &SlotIndex| {
            index
                .named
                .inspect_groups()
                .unwrap()
                .into_iter()
                .map(|group| (group.kind, group.scope, group.value, group.members))
                .collect::<Vec<_>>()
        };
        assert_eq!(groups(actual), groups(before));
    }
    fn limits() -> DomLimits {
        DomLimits::new(
            2048,
            1_000_000,
            8192,
            8192,
            0,
            8,
            64_000_000,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
    }
    fn fixture() -> (Dom, NodeId, [NodeId; 4]) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(limits()).unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let root = b.create_element_ns(&d, crate::Namespace::html(), "slot");
        let a = b.create_element_ns(&d, crate::Namespace::html(), "slot");
        let wrapper = b.create_element(&d, "wrapper");
        let nested = b.create_element_ns(&d, crate::Namespace::html(), "slot");
        let z = b.create_element_ns(&d, crate::Namespace::html(), "slot");
        let comment = b.create_comment(&d, "gap");
        b.insert_before(&nested, &wrapper, None);
        b.insert_before(&a, &root, None);
        b.insert_before(&comment, &root, None);
        b.insert_before(&wrapper, &root, None);
        b.insert_before(&z, &root, None);
        let c = dom.apply(b, &cancel).unwrap();
        let id = |t: &CreatedNode| c.created(t).unwrap().unwrap();
        let ids = [id(&root), id(&a), id(&nested), id(&z)];
        (dom, ids[0], ids)
    }
    fn source(dom: &Dom, cancel: &AtomicBool) -> Draft {
        let b = DomBatch::new();
        Draft::new_source(
            dom,
            b.owner,
            &b.commands,
            crate::DomSourceLimits::new(500_000_000, 10_000_000, 100_000_000, 64, 4096, 4096, 1),
            cancel,
        )
        .unwrap()
        .0
    }
    fn selected(draft: &Draft, root: Key, cancel: &AtomicBool) -> Vec<Key> {
        let mut result = Vec::new();
        let mut key = draft
            .slot_index
            .first(root, &mut || draft.source_step(cancel))
            .unwrap();
        while let Some(current) = key {
            result.push(current);
            key = draft
                .slot_index
                .next(root, current, &mut || draft.source_step(cancel))
                .unwrap();
        }
        result
    }
    #[test]
    fn each_assignment_invocation_includes_root_slot_then_ordinary_preorder() {
        let (dom, root, ids) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = source(&dom, &cancel);
        let root = Key::Existing(root);
        let expected: Vec<_> = ids.into_iter().map(Key::Existing).collect();
        for _ in 0..2 {
            let observed = super::super::super::slot::assignment_observation::Observation::begin();
            let measurement =
                scale_metrics::ENABLED.then(|| scale_metrics::Measurement::begin().unwrap());
            draft.assign_tree(root, dom.limits(), &cancel).unwrap();
            assert_eq!(observed.finish(), expected);
            if let Some(measurement) = measurement {
                let counters = measurement
                    .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Source))
                    .unwrap();
                assert_eq!(counters.slot_assignments, 4);
                assert!(counters.slot_discovery_steps > 0);
            }
        }
    }
    #[test]
    fn source_nested_relation_reorders_restore_immediate_positions_and_participants() {
        let (dom, root, ids) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = source(&dom, &cancel);
        let root = Key::Existing(root);
        let original = draft
            .node(root)
            .unwrap()
            .children
            .iter()
            .collect::<Vec<_>>();
        let outer = draft
            .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
            .unwrap();
        let mut reversed = original.clone();
        reversed.reverse();
        draft
            .relations_replace(
                root,
                super::super::RelationField::Children,
                reversed.clone(),
                &cancel,
            )
            .unwrap();
        assert_eq!(
            selected(&draft, root, &cancel),
            [ids[0], ids[3], ids[2], ids[1]].map(Key::Existing)
        );
        let inner = draft
            .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
            .unwrap();
        draft
            .relations_replace(
                root,
                super::super::RelationField::Children,
                original.clone(),
                &cancel,
            )
            .unwrap();
        assert_eq!(selected(&draft, root, &cancel), ids.map(Key::Existing));
        let spent = draft.source_work.load(Ordering::Relaxed);
        inner.restore(&mut draft);
        assert_eq!(
            selected(&draft, root, &cancel),
            [ids[0], ids[3], ids[2], ids[1]].map(Key::Existing)
        );
        outer.restore(&mut draft);
        assert_eq!(selected(&draft, root, &cancel), ids.map(Key::Existing));
        assert!(draft.source_work.load(Ordering::Relaxed) >= spent);
        assert_eq!(
            draft
                .node(root)
                .unwrap()
                .children
                .iter()
                .collect::<Vec<_>>(),
            original
        );
    }
    #[test]
    fn source_slot_plan_refusal_and_cancellation_preserve_actual_index() {
        let (dom, root, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = source(&dom, &cancel);
        let root = Key::Existing(root);
        let before = draft.slot_index.current.index.clone();
        let retained = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
        draft.source_limits.as_mut().unwrap().retained_bytes = retained;
        let work = draft.source_work.load(Ordering::Relaxed);
        let old = draft
            .node(root)
            .unwrap()
            .children
            .iter()
            .collect::<Vec<_>>();
        let mut new = old.clone();
        new.reverse();
        assert!(matches!(
            draft.prepare_slot_children_replace(root, &new, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert_unchanged_index(&draft.slot_index.current.index, &before);
        assert!(draft.source_work.load(Ordering::Relaxed) > work);
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            draft.prepare_slot_children_replace(root, &new, &cancel),
            Err(DomError::Cancelled)
        ));
        assert_unchanged_index(&draft.slot_index.current.index, &before);
    }

    #[test]
    fn ordinary_successor_reads_each_completed_ancestor_edge_once() {
        let (dom, root, ids) = fixture();
        let cancel = AtomicBool::new(false);
        let draft = source(&dom, &cancel);
        let root = Key::Existing(root);
        let children = draft.node(root).unwrap().children;
        let expected = [
            root,
            Key::Existing(ids[1]),
            children.get(1).unwrap(),
            children.get(2).unwrap(),
            Key::Existing(ids[2]),
            Key::Existing(ids[3]),
        ];
        let measurement =
            scale_metrics::ENABLED.then(|| scale_metrics::Measurement::begin().unwrap());
        let mut actual = Vec::new();
        let mut current = Some(root);
        while let Some(key) = current {
            actual.push(key);
            current = crate::slot::next(&draft, root, key, &cancel).unwrap();
        }
        assert_eq!(actual, expected);
        if let Some(measurement) = measurement {
            let counters = measurement
                .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Source))
                .unwrap();
            assert_eq!(counters.slot_ordinal_successor_reads, expected.len() - 1);
        }
    }

    #[test]
    fn first_position_preallocation_admits_its_exact_extent_and_refuses_one_byte_short() {
        let (dom, root, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = source(&dom, &cancel);
        let root = Key::Existing(root);
        let document = draft.node(root).unwrap().document;
        let child = draft
            .allocate(
                DomNodeKind::Comment("new".into()),
                document,
                limits(),
                &cancel,
            )
            .unwrap();
        assert!(!draft.journal.has_marks());
        let index = draft.node(root).unwrap().children.len();
        let before = draft.slot_index.current.index.clone();
        draft.custody_registry().reconcile_releases();
        let ledger = draft.custody_ledger().clone();
        let baseline = ledger.total();
        let external = custody::Usage::new(draft.source_other_bytes, draft.source_other_units);
        let total = baseline.checked_add(external).unwrap();
        // The first real retained allocation gate, not an estimate of later
        // registry/path peak. The semantic body allocates afterward.
        let body = custody::Usage::new(size_of::<ChildPosition>(), 0)
            .checked_add(custody::SharedDebit::administration_usage())
            .unwrap();
        let exact = total.checked_add(body).unwrap().bytes();
        for limit in [exact - 1, exact] {
            draft.source_limits.as_mut().unwrap().retained_bytes = limit;
            let measurement =
                scale_metrics::ENABLED.then(|| scale_metrics::Measurement::begin().unwrap());
            if measurement.is_some() {
                scale_metrics::select_source_ledger(
                    ledger.measurement_identity(),
                    baseline,
                    external,
                );
            }
            let work = draft.source_work.load(Ordering::Relaxed);
            assert!(
                matches!(draft.prepare_slot_children(root, index, 0, &[child], &cancel), Err(DomError::ResourceLimit { resource:DomResource::SourceRetainedBytes, limit:actual }) if actual==limit)
            );
            assert_unchanged_index(&draft.slot_index.current.index, &before);
            draft.custody_registry().reconcile_releases();
            assert_eq!(ledger.total(), baseline);
            assert!(draft.source_work.load(Ordering::Relaxed) > work);
            if let Some(measurement) = measurement {
                let counters = measurement
                    .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Accounting))
                    .unwrap();
                assert_eq!(counters.record_materializations, 0);
                if limit == exact - 1 {
                    assert_eq!(counters.structural_allocations, 0);
                    assert_eq!(counters.source_peak_bytes, total.bytes());
                } else {
                    // The exact first gate succeeds. Additional registry
                    // metadata correctly refuses at this same finite limit.
                    assert_eq!(counters.structural_allocations, 1);
                    assert_eq!(counters.structural_bytes, size_of::<custody::Debit>());
                    assert_eq!(counters.source_peak_bytes, exact);
                }
            }
        }
        draft.source_limits.as_mut().unwrap().retained_bytes = 500_000_000;
        let measurement =
            scale_metrics::ENABLED.then(|| scale_metrics::Measurement::begin().unwrap());
        if measurement.is_some() {
            scale_metrics::select_source_ledger(ledger.measurement_identity(), baseline, external);
        }
        let plan = draft
            .prepare_slot_children(root, index, 0, &[child], &cancel)
            .unwrap()
            .unwrap();
        assert_eq!(
            plan.next.index.position(child),
            Some(ChildPosition {
                key: child,
                parent: root,
                ordinal: index
            })
        );
        assert_unchanged_index(&draft.slot_index.current.index, &before);
        if let Some(measurement) = measurement {
            let counters = measurement
                .finish(scale_metrics::Hooks::one(scale_metrics::Domain::Accounting))
                .unwrap();
            assert!(counters.source_peak_bytes > exact);
            eprintln!(
                "actual complete SlotPlan peak={} first_gate_exact={exact}; whole-plan exact capacity is not claimed",
                counters.source_peak_bytes
            );
        }
        drop(plan);
        draft.custody_registry().reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }

    fn publish_slot_phase(draft: &mut Draft, dom: &mut Dom, cancel: &AtomicBool) {
        let prepared = draft.finish_segment(dom, cancel).unwrap();
        let mut publication = dom.prepare_publication(prepared, cancel).unwrap();
        let rebase = draft.prepare_rebase(&mut publication, cancel).unwrap();
        let sharing = draft
            .prepare_sharing_publication(publication.changed, None, cancel)
            .unwrap();
        draft.close_sharing_publication(&mut publication, sharing, None);
        let committed = dom.publish_prepared_with_proof(publication);
        draft.commit_publication(rebase, &committed.committed);
    }

    #[test]
    fn hidden_slot_exposure_preserves_original_created_alias_after_two_publications() {
        let cancel = AtomicBool::new(false);
        let (mut dom, root, ids) = fixture();
        let mut draft = source(&dom, &cancel);
        let root = Key::Existing(root);
        let document = draft.node(root).unwrap().document;
        let kind = draft.node(root).unwrap().kind.clone();
        let target = draft.allocate(kind, document, limits(), &cancel).unwrap();
        draft.mark_source_private(target, &cancel).unwrap();
        draft
            .set_attribute(
                root,
                None,
                &"phase".into(),
                &"hidden".into(),
                limits(),
                &cancel,
            )
            .unwrap();
        let before = dom.snapshot();
        publish_slot_phase(&mut draft, &mut dom, &cancel);
        assert!(draft.is_private(target));
        assert!(draft.published_id(target).is_none());
        assert!(dom.snapshot().revision() > before.revision());
        assert_eq!(dom.state.slot_index.inspect_members().len(), 4);
        assert_eq!(selected(&draft, target, &cancel), [target]);

        draft.expose_source_subtree(target, &cancel).unwrap();
        draft
            .pre_insert(target, root, Some(Key::Existing(ids[1])), limits(), &cancel)
            .unwrap();
        publish_slot_phase(&mut draft, &mut dom, &cancel);
        let published = draft.published_id(target).unwrap();
        assert_eq!(dom.state.slot_index.inspect_members().len(), 5);
        assert_eq!(
            selected(&draft, root, &cancel),
            [
                root,
                target,
                Key::Existing(ids[1]),
                Key::Existing(ids[2]),
                Key::Existing(ids[3])
            ]
        );
        let snapshot = dom.snapshot();
        assert_eq!(
            snapshot.node(published).unwrap().parent(),
            match root {
                Key::Existing(id) => Some(id),
                _ => unreachable!(),
            }
        );
        let mark = draft
            .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
            .unwrap();
        draft
            .pre_insert(Key::Existing(published), root, None, limits(), &cancel)
            .unwrap();
        assert_eq!(
            selected(&draft, root, &cancel),
            [
                root,
                Key::Existing(ids[1]),
                Key::Existing(ids[2]),
                Key::Existing(ids[3]),
                target
            ]
        );
        let spent = draft.source_work.load(Ordering::Relaxed);
        mark.restore(&mut draft);
        assert_eq!(
            selected(&draft, root, &cancel),
            [
                root,
                target,
                Key::Existing(ids[1]),
                Key::Existing(ids[2]),
                Key::Existing(ids[3])
            ]
        );
        assert!(draft.source_work.load(Ordering::Relaxed) >= spent);
        assert_eq!(
            snapshot.node(published).unwrap().parent(),
            dom.snapshot().node(published).unwrap().parent()
        );
    }

    #[test]
    #[ignore = "explicit release actual public SourceImport Slot counter geometry"]
    #[expect(
        clippy::assertions_on_constants,
        reason = "This ignored measurement requires the actual release/counter build"
    )]
    fn actual_source_import_slot_participant_geometry() {
        use crate::{DomSourceEdit, DomSourceLimits, DomSourceOutput, DomSourceProgress};
        assert!(!cfg!(debug_assertions), "release build required");
        assert!(scale_metrics::ENABLED, "actual owning counters required");
        let cancel = AtomicBool::new(false);
        for comments in [8usize, 32, 128, 512] {
            for slots in [0usize, 2, comments / 8] {
                let mut input = Dom::new(limits()).unwrap();
                let mut b = DomBatch::new();
                let d = b.create_document();
                let root = b.create_element(&d, "captured");
                let mut shape = Vec::new();
                for index in 0..comments {
                    if slots != 0 && index % (comments / slots) == 0 {
                        let slot = b.create_element_ns(&d, crate::Namespace::html(), "slot");
                        b.set_attribute(&slot, "name", "same");
                        b.insert_before(&slot, &root, None);
                        shape.push(true);
                    }
                    let c = b.create_comment(&d, "payload");
                    b.insert_before(&c, &root, None);
                    shape.push(false);
                }
                let c = input.apply(b, &cancel).unwrap();
                let root = c.created(&root).unwrap().unwrap();
                let captured = input.snapshot();
                drop(input);
                let mut target = Dom::new(limits()).unwrap();
                let mut b = DomBatch::new();
                let d = b.create_document();
                let c = target.apply(b, &cancel).unwrap();
                let document = c.created(&d).unwrap().unwrap();
                let before = target.snapshot();
                let measurement = scale_metrics::Measurement::begin().unwrap();
                let mut lease = target
                    .begin_source_edit(
                        DomSourceEdit::import_node(&captured, root, document, true),
                        DomSourceLimits::new(
                            500_000_000,
                            10_000_000,
                            if slots <= 2 { 2_000_000 } else { 100_000_000 },
                            8,
                            4096,
                            4096,
                            1,
                        ),
                        &cancel,
                    )
                    .unwrap();
                let DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
                    panic!("generic XML/Slot clone needs no external service");
                };
                let imported = done.published_node().unwrap();
                assert_eq!(done.output(), Some(DomSourceOutput::Node(imported)));
                drop(lease);
                let counters = measurement
                    .finish(
                        scale_metrics::Hooks::one(scale_metrics::Domain::Source)
                            .union(scale_metrics::Hooks::one(scale_metrics::Domain::Bank))
                            .union(scale_metrics::Hooks::one(scale_metrics::Domain::Accounting))
                            .union(scale_metrics::Hooks::one(
                                scale_metrics::Domain::Transaction,
                            )),
                    )
                    .unwrap();
                let records = comments + slots + 1;
                let depth = (usize::BITS - records.leading_zeros()) as usize;
                // At most a fixed number of indexed owner-hook paths per created
                // record, with bounded radix/AVL depth. Actual Slot assignment may
                // visit all P participants at each of the k source hooks.
                let path_bound = (records + 8) * (slots + 1) * (depth + 1) * 64;
                assert!(
                    counters.slot_discovery_steps <= path_bound,
                    "comments={comments} slots={slots} {counters:?}"
                );
                assert!(
                    counters.slot_assignments <= (records + 8) * (slots + 1) * 8,
                    "{counters:?}"
                );
                assert!(
                    counters.slot_ordinal_successor_reads <= (records + 8) * (slots + 1) * 8,
                    "{counters:?}"
                );
                if slots == 0 {
                    assert_eq!(counters.slot_assignments, 0);
                } else {
                    assert!(counters.slot_assignments >= slots);
                }
                let now = target.snapshot();
                assert!(now.revision() > before.revision());
                assert_eq!(before.nodes().count(), 1);
                assert_eq!(now.nodes().count(), records + 1);
                assert_eq!(captured.node(root).unwrap().children().len(), shape.len());
                let node = now.node(imported).unwrap();
                assert_eq!(node.owner_document().unwrap().node(), document);
                assert_eq!(node.parent(), None);
                assert_eq!(node.children().len(), shape.len());
                let mut expected_assignments = Vec::new();
                for (&child, slot) in node.children().iter().zip(shape) {
                    let n = now.node(child).unwrap();
                    assert_eq!(n.parent(), Some(imported));
                    if slot {
                        expected_assignments.push(Key::Existing(child));
                        assert!(now.slot(child).unwrap().assigned_nodes().is_empty());
                    } else {
                        assert!(matches!(n.kind(), DomNodeKind::Comment(_)));
                        assert_eq!(n.data(), Some(&"payload".into()));
                    }
                }
                assert!(now.pending_slot_signals().is_empty());
                // Outside the measurement window, independently check the
                // actual selected assignment order against the literal input
                // pattern and published IDs, including the growing-P case.
                let mut validation = source(&target, &cancel);
                let observed =
                    super::super::super::slot::assignment_observation::Observation::begin();
                validation
                    .assign_tree(Key::Existing(imported), limits(), &cancel)
                    .unwrap();
                assert_eq!(observed.finish(), expected_assignments);
                eprintln!(
                    "actual public SourceImport Slot counters comments={comments} slots={slots} path_bound={path_bound} {counters:?}"
                );
            }
        }
    }
}
