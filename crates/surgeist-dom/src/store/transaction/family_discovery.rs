//! Derived ordered participants for the existing Details and Select owners.
//! Structural membership changes here; HTML-owned state changes remain in the
//! original native hooks. In particular, this owner never writes option caches
//! or derives a new Selectedcontent disabled bit.
use super::super::*;
use super::exact_forest::{self, Forest, KeyView, Suffix};
use super::ordered_members::{
    Edit, Map, MapCustody, Members, MembersCustody, Membership, Mode, OrderPair, OrderView,
    Qualified, RootHeader,
};
use super::{NodeChange, RelationChange, RelationField};
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Details,
    Option,
    Target,
}
#[derive(Clone, Debug)]
struct Fact {
    key: Key,
    kind: Kind,
    root: Key,
    name: Option<DocumentString>,
    select: Option<Key>,
}
impl Qualified for Fact {
    fn key(&self) -> Key {
        self.key
    }
    fn usage(&self) -> Result<custody::Usage, DomError> {
        let mut payload = source::storage::Payload {
            bytes: size_of::<Self>(),
            units: 0,
        };
        if let Some(name) = &self.name {
            payload.string(name)?;
        }
        Ok(custody::Usage::new(payload.bytes, payload.units))
    }
}
impl PartialEq for Fact {
    fn eq(&self, other: &Self) -> bool {
        self.same(other)
    }
}
impl Eq for Fact {}
impl Fact {
    fn same(&self, other: &Self) -> bool {
        self.key == other.key
            && self.kind == other.kind
            && self.root == other.root
            && self.select == other.select
            && match (&self.name, &other.name) {
                (None, None) => true,
                (Some(a), Some(b)) => Arc::ptr_eq(a.shared_code_units(), b.shared_code_units()),
                _ => false,
            }
    }
    fn named_key(&self) -> Result<KeyView<'_, 3>, DomError> {
        Ok(KeyView {
            words: key_words(self.root),
            used: 3,
            suffix: Suffix::Utf16(
                self.name
                    .as_ref()
                    .ok_or(DomError::SelectProtocol)?
                    .code_units(),
            ),
        })
    }
}
fn key_words(key: Key) -> [u64; 3] {
    match key {
        Key::Existing(id) => [0, id.slot as u64, id.generation],
        Key::Created(index) => [1, index as u64, 0],
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Group {
    key: Key,
    members: RootHeader,
}
impl Qualified for Group {
    fn key(&self) -> Key {
        self.key
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NamedGroup {
    representative: Key,
    members: RootHeader,
}

/// Canonical derived values carry no ownership permission.
#[derive(Clone, Debug, Default)]
pub(in crate::store) struct FamilyIndex {
    owner: Option<DomOwnerId>,
    facts: Map<Fact>,
    details: Forest<NamedGroup>,
    details_root: exact_forest::Root,
    options: Map<Group>,
    targets: Map<Group>,
    members: Members,
}
#[derive(Clone, Default)]
pub(in crate::store) struct FamilyCustody {
    facts: MapCustody<Fact>,
    details: exact_forest::Custody<NamedGroup>,
    options: MapCustody<Group>,
    targets: MapCustody<Group>,
    members: MembersCustody,
}
#[derive(Clone, Default)]
struct Version {
    index: FamilyIndex,
    custody: FamilyCustody,
    dirty: Map<Key>,
    dirty_custody: MapCustody<Key>,
}
pub(in crate::store) struct Families {
    base: FamilyIndex,
    base_custody: FamilyCustody,
    current: Version,
}
pub(in crate::store) struct FamilyPlan {
    next: Version,
}
pub(in crate::store) struct FamilyUndo {
    before: Version,
}
pub(in crate::store) struct Finished {
    pub value: FamilyIndex,
    pub custody: FamilyCustody,
}
impl FamilyPlan {
    pub fn commit(self, families: &mut Families) -> FamilyUndo {
        FamilyUndo {
            before: std::mem::replace(&mut families.current, self.next),
        }
    }
}
impl FamilyUndo {
    pub fn restore(self, families: &mut Families) {
        families.current = self.before;
    }
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        0
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
        self.0.reconcile_releases();
    }
}

impl Families {
    pub fn capture(base: &FamilyIndex, custody: &FamilyCustody) -> Self {
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
    pub fn mark_published(&mut self, base: &FamilyIndex, custody: &FamilyCustody) {
        self.base = base.clone();
        self.base_custody = custody.clone();
        self.current.dirty = Default::default();
        self.current.dirty_custody = Default::default();
    }
    pub fn details(
        &self,
        root: Key,
        name: &DocumentString,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<RootHeader, DomError> {
        Ok(self
            .current
            .index
            .named_group(root, name, step)?
            .map_or_else(RootHeader::default, |group| group.members))
    }
    pub fn options(&self, select: Key) -> RootHeader {
        self.current
            .index
            .options
            .get(select)
            .map_or_else(RootHeader::default, |group| group.members)
    }
    pub fn targets(&self, select: Key) -> RootHeader {
        self.current
            .index
            .targets
            .get(select)
            .map_or_else(RootHeader::default, |group| group.members)
    }
    pub fn first(
        &self,
        root: RootHeader,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.current.index.members.first(root, step)
    }
    pub fn after(
        &self,
        root: RootHeader,
        key: Key,
        order: OrderView<'_>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.current
            .index
            .members
            .after(root, key, order, Mode::Ordinary, step)
    }
    pub fn finish(
        &self,
        draft: &Draft,
        mapper: &impl Fn(Key) -> Option<NodeId>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        let _cleanup = Reconcile(draft.custody_registry().clone());
        let mut output = Version {
            index: self.base.clone(),
            custody: self.base_custody.clone(),
            ..Default::default()
        };
        output.index.owner = Some(draft.store);
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        // Mapper retains the original qualified identity for retirements; a
        // never-published private creation has no canonical entry to remove.
        for &key in self.current.dirty.values() {
            (edit.step)()?;
            if let Some(id) = mapper(key)
                && let Some(previous) = self.base.facts.get(Key::Existing(id))
            {
                output.membership(previous, &self.base.facts, false, order.before, &mut edit)?;
            }
        }
        for &key in self.current.dirty.values() {
            (edit.step)()?;
            if let Some(id) = mapper(key) {
                let public = Key::Existing(id);
                let value = if self.current.index.facts.get(draft.live_key(key)).is_some() {
                    let name = self
                        .current
                        .index
                        .facts
                        .get(draft.live_key(key))
                        .and_then(|fact| fact.name.as_ref());
                    fact(order.after, public, Name::Cached(name), &mut *edit.step)?
                } else {
                    None
                };
                output
                    .index
                    .facts
                    .change(&mut output.custody.facts, public, value, &mut edit)?;
            }
        }
        let facts = output.index.facts.clone();
        for &key in self.current.dirty.values() {
            (edit.step)()?;
            if let Some(id) = mapper(key)
                && let Some(value) = facts.get(Key::Existing(id))
            {
                output.membership(value, &facts, true, order.after, &mut edit)?;
            }
        }
        Ok(Finished {
            value: output.index,
            custody: output.custody,
        })
    }
}

fn tag(kind: &DomNodeKind) -> Option<surgeist_html::KnownHtmlElement> {
    match kind {
        DomNodeKind::Element(element) => surgeist_html::KnownHtmlElement::from_name(element.name()),
        _ => None,
    }
}
enum Name<'a> {
    Read,
    Cached(Option<&'a DocumentString>),
}
struct NodeFact {
    key: Key,
    value: Option<Fact>,
}
fn fact(
    order: OrderView<'_>,
    key: Key,
    name: Name<'_>,
    step: &mut dyn FnMut() -> Result<(), DomError>,
) -> Result<Option<Fact>, DomError> {
    use surgeist_html::KnownHtmlElement as Tag;
    use surgeist_html::bounded::{
        HtmlOptionSelectAncestorAction as Action, HtmlOptionSelectAncestors,
    };
    let key = order.normalize(key);
    let node = order.node(key)?;
    let DomNodeKind::Element(element) = node.kind else {
        return Ok(None);
    };
    let mut value = Fact {
        key,
        kind: Kind::Option,
        root: key,
        name: None,
        select: None,
    };
    match Tag::from_name(element.name()) {
        Some(Tag::Details) => {
            let name = match name {
                Name::Cached(name) => name,
                Name::Read => {
                    let mut name = None;
                    for attribute in element.attributes().iter() {
                        step()?;
                        if attribute.name().namespace_uri().is_none()
                            && attribute
                                .name()
                                .local_name()
                                .code_units()
                                .iter()
                                .copied()
                                .eq("name".encode_utf16())
                        {
                            name = Some(attribute.value());
                            break;
                        }
                    }
                    name
                }
            };
            let Some(name) = name.filter(|name| !name.is_empty()) else {
                return Ok(None);
            };
            value.kind = Kind::Details;
            value.root = order.root(key, Mode::Ordinary, step)?;
            value.name = Some(name.clone());
        }
        Some(Tag::Option) => {
            let mut cursor = HtmlOptionSelectAncestors::default();
            let mut next = order.parent(key, Mode::Ordinary, step)?;
            while let Some(parent) = next {
                step()?;
                match cursor.observe(tag(order.node(parent.parent)?.kind)) {
                    Action::NearestSelect => {
                        value.select = Some(parent.parent);
                        break;
                    }
                    Action::NoSelect => break,
                    Action::Continue => next = order.parent(parent.parent, Mode::Ordinary, step)?,
                }
            }
        }
        Some(Tag::Selectedcontent) if element.selectedcontent_disabled() == Some(false) => {
            value.kind = Kind::Target
        }
        _ => return Ok(None),
    }
    Ok(Some(value))
}

impl FamilyIndex {
    fn qualified(&self, key: Key) -> Result<(), DomError> {
        if let Key::Existing(id) = key
            && self.owner.is_some_and(|owner| owner != id.owner)
        {
            return Err(DomError::ForeignOwner);
        }
        Ok(())
    }
    fn named(facts: &Map<Fact>, group: NamedGroup) -> Result<KeyView<'_, 3>, DomError> {
        facts
            .get(group.representative)
            .ok_or(DomError::SelectProtocol)?
            .named_key()
    }
    fn named_group(
        &self,
        root: Key,
        name: &DocumentString,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<NamedGroup>, DomError> {
        self.qualified(root)?;
        let key = KeyView {
            words: key_words(root),
            used: 3,
            suffix: Suffix::Utf16(name.code_units()),
        };
        self.details.lookup(
            self.details_root,
            &key,
            |group| Self::named(&self.facts, group),
            step,
        )
    }
    #[cfg(test)]
    pub fn heap_usage(&self) -> Result<custody::Usage, DomError> {
        self.facts
            .heap_usage()?
            .checked_add(self.details.heap_usage()?)?
            .checked_add(self.options.heap_usage()?)?
            .checked_add(self.targets.heap_usage()?)?
            .checked_add(self.members.heap_usage()?)
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<FamilyCustody, DomError> {
        let facts = self.facts.adopt_custody(registry, &mut *admit, step)?;
        let details = self.details.adopt_custody(registry, &mut *admit, step)?;
        let options = self.options.adopt_custody(registry, &mut *admit, step)?;
        let targets = self.targets.adopt_custody(registry, &mut *admit, step)?;
        let members = self.members.adopt_custody(registry, admit, step)?;
        Ok(FamilyCustody {
            facts,
            details,
            options,
            targets,
            members,
        })
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let mut graph = self.facts.inspect_graph()?;
        graph.merge(&self.details.inspect_graph()?)?;
        graph.merge(&self.options.inspect_graph()?)?;
        graph.merge(&self.targets.inspect_graph()?)?;
        graph.merge(&self.members.inspect_graph()?)?;
        Ok(graph)
    }
    #[cfg(test)]
    pub fn inspect_facts(&self) -> Vec<FactInspection> {
        self.facts
            .values()
            .map(|fact| FactInspection {
                key: fact.key,
                kind: fact.kind.into(),
                root: fact.root,
                name: fact.name.clone(),
                select: fact.select,
            })
            .collect()
    }
    #[cfg(test)]
    pub fn inspect_groups(&self) -> Result<Vec<GroupInspection>, DomError> {
        let mut groups = Vec::new();
        let mut pending = Vec::new();
        pending.extend(self.details_root);
        while let Some(index) = pending.pop() {
            match self.details.node(index).ok_or(DomError::SelectProtocol)? {
                exact_forest::Node::Branch { children, .. } => pending.extend(children),
                exact_forest::Node::Leaf(group) => {
                    let fact = self
                        .facts
                        .get(group.representative)
                        .ok_or(DomError::SelectProtocol)?;
                    groups.push(GroupInspection {
                        kind: FactKindInspection::Details,
                        owner: fact.root,
                        name: fact.name.clone(),
                        members: self.members.inspect_group(group.members)?,
                    });
                }
            }
        }
        for (kind, map) in [
            (FactKindInspection::Option, &self.options),
            (FactKindInspection::Target, &self.targets),
        ] {
            for group in map.values() {
                groups.push(GroupInspection {
                    kind,
                    owner: group.key,
                    name: None,
                    members: self.members.inspect_group(group.members)?,
                });
            }
        }
        Ok(groups)
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum FactKindInspection {
    Details,
    Option,
    Target,
}
#[cfg(test)]
impl From<Kind> for FactKindInspection {
    fn from(value: Kind) -> Self {
        match value {
            Kind::Details => Self::Details,
            Kind::Option => Self::Option,
            Kind::Target => Self::Target,
        }
    }
}
#[cfg(test)]
pub(in crate::store) struct FactInspection {
    pub key: Key,
    pub kind: FactKindInspection,
    pub root: Key,
    pub name: Option<DocumentString>,
    pub select: Option<Key>,
}
#[cfg(test)]
pub(in crate::store) struct GroupInspection {
    pub kind: FactKindInspection,
    pub owner: Key,
    pub name: Option<DocumentString>,
    pub members: Vec<Key>,
}

impl Version {
    fn member(
        &mut self,
        root: RootHeader,
        key: Key,
        present: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<RootHeader, DomError> {
        let changed = self.index.members.change(
            &self.custody.members,
            root,
            if present {
                Membership::Insert(key)
            } else {
                Membership::Remove(key)
            },
            order,
            Mode::Ordinary,
            Edit {
                registry: edit.registry,
                admit: &mut *edit.admit,
                step: &mut *edit.step,
            },
        )?;
        self.index.members = changed.members;
        self.custody.members = changed.custody;
        Ok(changed.root)
    }
    fn selected_group(
        &mut self,
        group: Key,
        key: Key,
        present: bool,
        target: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        let old = if target {
            self.index.targets.get(group)
        } else {
            self.index.options.get(group)
        }
        .copied();
        if old.is_none() && !present {
            return Ok(());
        }
        let members = self.member(
            old.map_or_else(RootHeader::default, |row| row.members),
            key,
            present,
            order,
            edit,
        )?;
        let row = (!members.is_empty()).then_some(Group {
            key: group,
            members,
        });
        if target {
            self.index
                .targets
                .change(&mut self.custody.targets, group, row, edit)?;
        } else {
            self.index
                .options
                .change(&mut self.custody.options, group, row, edit)?;
        }
        Ok(())
    }
    fn named_member(
        &mut self,
        fact: &Fact,
        facts: &Map<Fact>,
        present: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        let key = fact.named_key()?;
        let group = self.index.details.lookup(
            self.index.details_root,
            &key,
            |group| FamilyIndex::named(facts, group),
            &mut *edit.step,
        )?;
        if group.is_none() && !present {
            return Ok(());
        }
        let members = self.member(
            group.map_or_else(RootHeader::default, |group| group.members),
            fact.key,
            present,
            order,
            edit,
        )?;
        let value = if members.is_empty() {
            None
        } else {
            let representative = if let Some(group) =
                group.filter(|group| group.representative != fact.key || present)
            {
                group.representative
            } else {
                self.index
                    .members
                    .first(members, &mut *edit.step)?
                    .ok_or(DomError::SelectProtocol)?
            };
            Some(NamedGroup {
                representative,
                members,
            })
        };
        let changed = self.index.details.change(
            &self.custody.details,
            self.index.details_root,
            &key,
            value,
            |group| FamilyIndex::named(facts, group),
            exact_forest::Edit {
                registry: edit.registry,
                admit: &mut *edit.admit,
                step: &mut *edit.step,
            },
        )?;
        self.index.details = changed.forest;
        self.custody.details = changed.custody;
        self.index.details_root = changed.root;
        Ok(())
    }
    fn membership(
        &mut self,
        fact: &Fact,
        facts: &Map<Fact>,
        present: bool,
        order: OrderView<'_>,
        edit: &mut Edit<'_>,
    ) -> Result<(), DomError> {
        match fact.kind {
            Kind::Details => self.named_member(fact, facts, present, order, edit),
            Kind::Option => {
                if let Some(select) = fact.select {
                    self.selected_group(select, fact.key, present, false, order, edit)?;
                }
                Ok(())
            }
            Kind::Target => {
                let mut next = order.parent(fact.key, Mode::Ordinary, &mut *edit.step)?;
                while let Some(parent) = next {
                    if tag(order.node(parent.parent)?.kind)
                        == Some(surgeist_html::KnownHtmlElement::Select)
                    {
                        self.selected_group(parent.parent, fact.key, present, true, order, edit)?;
                    }
                    next = order.parent(parent.parent, Mode::Ordinary, &mut *edit.step)?;
                }
                Ok(())
            }
        }
    }
}

fn gather(
    draft: &Draft,
    root: Key,
    order: OrderView<'_>,
    keys: &mut Keys,
    edit: &mut Edit<'_>,
    cancel: &AtomicBool,
) -> Result<(), DomError> {
    let mut pending = source::scratch::ScratchVec::new(draft, false);
    pending.reserve(draft, 1, cancel)?;
    pending.push(root);
    while let Some(key) = pending.pop() {
        (edit.step)()?;
        let key = order.normalize(key);
        if keys.values.get(key).is_some() {
            continue;
        }
        keys.values
            .change(&mut keys.custody, key, Some(key), edit)?;
        let node = order.node(key)?;
        pending.reserve(draft, node.children.len(), cancel)?;
        pending.extend(node.children.iter());
    }
    Ok(())
}

impl Families {
    fn prepare(
        &self,
        draft: &Draft,
        keys: &Map<Key>,
        order: OrderPair<'_>,
        mut candidate: Option<NodeFact>,
        cancel: &AtomicBool,
    ) -> Result<Option<FamilyPlan>, DomError> {
        let _cleanup = Reconcile(draft.custody_registry().clone());
        let mut next = self.current.clone();
        next.index.owner = Some(draft.store);
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        let mut changed = false;
        // Removing the complete affected set before inserting under the new
        // comparator preserves both the old and new ordered group invariants.
        for &key in keys.values() {
            if let Some(previous) = self.current.index.facts.get(key) {
                next.membership(
                    previous,
                    &self.current.index.facts,
                    false,
                    order.before,
                    &mut edit,
                )?;
                changed = true;
            }
            let value = if candidate
                .as_ref()
                .is_some_and(|candidate| candidate.key == key)
            {
                candidate.take().expect("selected node fact").value
            } else {
                // A relation edit changes topology, never attributes. Reuse
                // the already classified name, including known absence.
                let name = self
                    .current
                    .index
                    .facts
                    .get(key)
                    .and_then(|fact| fact.name.as_ref());
                fact(order.after, key, Name::Cached(name), &mut *edit.step)?
            };
            if value.is_some() {
                changed = true;
            }
            next.index
                .facts
                .change(&mut next.custody.facts, key, value, &mut edit)?;
        }
        if !changed {
            return Ok(None);
        }
        let facts = next.index.facts.clone();
        for &key in keys.values() {
            if let Some(value) = facts.get(key) {
                next.membership(value, &facts, true, order.after, &mut edit)?;
            }
            if self.current.index.facts.get(key).is_some() || facts.get(key).is_some() {
                next.dirty
                    .change(&mut next.dirty_custody, key, Some(key), &mut edit)?;
            }
        }
        Ok(Some(FamilyPlan { next }))
    }
    pub fn prepare_node(
        &self,
        draft: &Draft,
        change: NodeChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<FamilyPlan>, DomError> {
        let key = draft.live_key(change.key);
        let unchanged_attributes = match (change.before, change.after) {
            (Some(before), Some(after)) => match (before.kind, after.kind) {
                (DomNodeKind::Element(before), DomNodeKind::Element(after)) => {
                    before
                        .attributes_backing()
                        .same_storage(after.attributes_backing())
                        && surgeist_html::KnownHtmlElement::from_name(before.name())
                            == surgeist_html::KnownHtmlElement::from_name(after.name())
                }
                _ => false,
            },
            _ => false,
        };
        let name = if unchanged_attributes {
            Name::Cached(
                self.current
                    .index
                    .facts
                    .get(key)
                    .and_then(|fact| fact.name.as_ref()),
            )
        } else {
            Name::Read
        };
        let after = if change.after.is_some() {
            fact(order.after, key, name, &mut || draft.source_step(cancel))?
        } else {
            None
        };
        let before = self.current.index.facts.get(key);
        let different_tag =
            change.before.map(|node| tag(node.kind)) != change.after.map(|node| tag(node.kind));
        let descendant_rule_changed =
            different_tag && change.before.is_some() && change.after.is_some();
        if !descendant_rule_changed && before == after.as_ref() {
            return Ok(None);
        }
        let _cleanup = Reconcile(draft.custody_registry().clone());
        let mut keys = Keys::default();
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        keys.values
            .change(&mut keys.custody, key, Some(key), &mut edit)?;
        if descendant_rule_changed {
            // A genuine ancestor-kind replacement may change HTML pruning for
            // actual descendants; ordinary scalar changes never take this path.
            let mut pending = source::scratch::ScratchVec::new(draft, false);
            let node = order.after.node(key)?;
            pending.reserve(draft, node.children.len(), cancel)?;
            pending.extend(node.children.iter());
            for child in pending.iter() {
                gather(draft, *child, order.after, &mut keys, &mut edit, cancel)?;
            }
        }
        self.prepare(
            draft,
            &keys.values,
            order,
            Some(NodeFact { key, value: after }),
            cancel,
        )
    }
    pub fn prepare_relation(
        &self,
        draft: &Draft,
        change: RelationChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<FamilyPlan>, DomError> {
        if !matches!(change.field, RelationField::Children) {
            return Ok(None);
        }
        let _cleanup = Reconcile(draft.custody_registry().clone());
        let mut keys = Keys::default();
        let mut edit = Edit {
            registry: draft.custody_registry(),
            admit: &mut |usage| draft.admit_custody_usage(usage, cancel),
            step: &mut || draft.source_step(cancel),
        };
        let end = change
            .index
            .checked_add(change.removed)
            .ok_or(DomError::RepresentationExhausted)?;
        for ordinal in change.index..end {
            let key = change.before.get(ordinal).ok_or(DomError::IndexSize)?;
            gather(draft, key, order.before, &mut keys, &mut edit, cancel)?;
        }
        for &key in change.added {
            gather(draft, key, order.after, &mut keys, &mut edit, cancel)?;
        }
        self.prepare(draft, &keys.values, order, None, cancel)
    }
    pub fn prepare_visibility(
        &self,
        draft: &Draft,
        key: Key,
        was_private: bool,
        is_private: bool,
        _order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<FamilyPlan>, DomError> {
        let key = draft.live_key(key);
        if was_private == is_private || self.current.index.facts.get(key).is_none() {
            return Ok(None);
        }
        let _cleanup = Reconcile(draft.custody_registry().clone());
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
        Ok(Some(FamilyPlan { next }))
    }
}
