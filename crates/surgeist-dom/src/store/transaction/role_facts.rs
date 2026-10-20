//! Exact relation roles and sparse direct-body facts. Canonical sequences remain
//! their semantic owners; this index records occurrences, never another order.
use super::super::*;
use super::KindRead;
use super::counted_relations::{Change, Changed, Custody as MapCustody, Delta, Index as Map};
use super::ordered_members::{OrderPair, OrderView};
use super::{NodeChange, NodeRead, RelationChange, RelationField, SequenceView};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum Role {
    RenderBlocking,
    AutofocusCandidates,
    OpenDialogs,
    TopLayer,
    PendingTopLayerRemovals,
    BodyChild,
    DocumentElement,
    ShowingPopover,
    FullscreenFlagged,
}
const ROLES: usize = 9;
impl Role {
    const ALL: [Self; ROLES] = [
        Self::RenderBlocking,
        Self::AutofocusCandidates,
        Self::OpenDialogs,
        Self::TopLayer,
        Self::PendingTopLayerRemovals,
        Self::BodyChild,
        Self::DocumentElement,
        Self::ShowingPopover,
        Self::FullscreenFlagged,
    ];
    fn index(self) -> usize {
        self as usize
    }
    fn relation(field: RelationField) -> Option<Self> {
        match field {
            RelationField::RenderBlocking => Some(Self::RenderBlocking),
            RelationField::AutofocusCandidates => Some(Self::AutofocusCandidates),
            RelationField::OpenDialogs => Some(Self::OpenDialogs),
            RelationField::TopLayer => Some(Self::TopLayer),
            RelationField::PendingTopLayerRemovals => Some(Self::PendingTopLayerRemovals),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(in crate::store) struct Index {
    forward: [Map; ROLES],
    reverse: [Map; ROLES],
    usage: custody::Usage,
}
#[derive(Clone, Default)]
pub(in crate::store) struct Custody {
    forward: [MapCustody; ROLES],
    reverse: [MapCustody; ROLES],
}
#[derive(Clone, Default)]
struct Dirty {
    value: [Map; ROLES],
    custody: [MapCustody; ROLES],
}
#[derive(Clone, Default)]
struct Version {
    value: Index,
    custody: Custody,
    dirty: Dirty,
}
#[derive(Clone, Default)]
pub(in crate::store) struct Roles {
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
impl Plan {
    pub fn commit(self, roles: &mut Roles) -> Undo {
        Undo {
            before: std::mem::replace(&mut roles.current, self.next),
        }
    }
}
impl Undo {
    pub fn restore(self, roles: &mut Roles) {
        roles.current = self.before;
    }
}

struct Editing<'a> {
    registry: &'a bank::CustodyRegistry,
    admit: &'a mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    step: &'a mut dyn FnMut() -> Result<(), DomError>,
    changed: bool,
}
fn difference(old: usize, new: usize) -> Delta {
    if new > old {
        Delta::Add(new - old)
    } else {
        Delta::Remove(old - new)
    }
}
fn replace_map(map: &mut Map, custody: &mut MapCustody, changed: Changed) {
    let previous = Changed {
        value: std::mem::replace(map, changed.value),
        custody: std::mem::replace(custody, changed.custody),
    };
    drop(previous);
}
impl Index {
    #[cfg(test)]
    pub fn retained_usage(&self) -> custody::Usage {
        self.usage
    }
    pub fn count(&self, role: Role, owner: Key, member: Key) -> usize {
        self.forward[role.index()].count(owner, member)
    }
    pub fn has_any(&self, role: Role, owner: Key) -> bool {
        self.forward[role.index()].has_group(owner)
    }
    pub fn visit_owners(
        &self,
        role: Role,
        member: Key,
        visit: &mut impl FnMut(Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.reverse[role.index()].visit_group(member, visit, step)
    }
    pub fn document_element(
        &self,
        document: Key,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        let mut element = None;
        self.forward[Role::DocumentElement.index()].visit_group(
            document,
            &mut |key, _| {
                // The ordinary insertion owner enforces one Element per Document.
                if element.replace(key).is_some() {
                    return Err(DomError::HierarchyRequest);
                }
                Ok(())
            },
            step,
        )?;
        Ok(element)
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Custody, DomError> {
        let mut custody = Custody::default();
        for role in Role::ALL {
            let i = role.index();
            custody.forward[i] = self.forward[i].adopt_custody(registry, admit, step)?;
            custody.reverse[i] = self.reverse[i].adopt_custody(registry, admit, step)?;
        }
        Ok(custody)
    }
    fn set(
        &mut self,
        custody: &mut Custody,
        role: Role,
        owner: Key,
        member: Key,
        new: usize,
        edit: &mut Editing<'_>,
    ) -> Result<bool, DomError> {
        let i = role.index();
        let old = self.forward[i].count(owner, member);
        if old == new {
            return Ok(false);
        }
        let forward = self.forward[i].change(
            &custody.forward[i],
            Change {
                group: owner,
                member,
                delta: difference(old, new),
            },
            edit.registry,
            &mut *edit.admit,
            &mut *edit.step,
        )?;
        let reverse = self.reverse[i].change(
            &custody.reverse[i],
            Change {
                group: member,
                member: owner,
                delta: difference(old, new),
            },
            edit.registry,
            &mut *edit.admit,
            &mut *edit.step,
        )?;
        let usage = self
            .usage
            .checked_sub(self.forward[i].retained_usage())?
            .checked_sub(self.reverse[i].retained_usage())?
            .checked_add(forward.value.retained_usage())?
            .checked_add(reverse.value.retained_usage())?;
        replace_map(&mut self.forward[i], &mut custody.forward[i], forward);
        replace_map(&mut self.reverse[i], &mut custody.reverse[i], reverse);
        self.usage = usage;
        edit.registry.reconcile_releases();
        Ok(true)
    }
}
impl Version {
    fn set(
        &mut self,
        role: Role,
        owner: Key,
        member: Key,
        new: usize,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        if !self
            .value
            .set(&mut self.custody, role, owner, member, new, edit)?
        {
            return Ok(());
        }
        self.mark(role, owner, member, edit)?;
        edit.changed = true;
        Ok(())
    }
    fn mark(
        &mut self,
        role: Role,
        owner: Key,
        member: Key,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        let i = role.index();
        if self.dirty.value[i].count(owner, member) == 0 {
            let next = self.dirty.value[i].change(
                &self.dirty.custody[i],
                Change {
                    group: owner,
                    member,
                    delta: Delta::Add(1),
                },
                edit.registry,
                &mut *edit.admit,
                &mut *edit.step,
            )?;
            replace_map(&mut self.dirty.value[i], &mut self.dirty.custody[i], next);
        }
        Ok(())
    }
    fn change(
        &mut self,
        role: Role,
        owner: Key,
        member: Key,
        add: bool,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        let old = self.value.count(role, owner, member);
        let new = if add {
            old.checked_add(1)
        } else {
            old.checked_sub(1)
        }
        .ok_or(DomError::RepresentationExhausted)?;
        self.set(role, owner, member, new, edit)
    }
    fn raw_change(
        &mut self,
        role: Role,
        owner: Key,
        member: Key,
        add: bool,
        node: NodeRead<'_>,
        edit: &mut Editing<'_>,
    ) -> Result<(), DomError> {
        self.change(role, owner, member, add, edit)?;
        if role == Role::TopLayer {
            if showing(node) {
                self.change(Role::ShowingPopover, owner, member, add, edit)?;
            }
            if flagged(node) {
                self.change(Role::FullscreenFlagged, owner, member, add, edit)?;
            }
        }
        Ok(())
    }
    fn remove_key(&mut self, key: Key, edit: &mut Editing<'_>) -> Result<(), DomError> {
        // Only actual role occurrences of this retired key are visited.
        let previous = self.clone();
        let work = std::cell::RefCell::new(&mut *edit.step);
        let admission = std::cell::RefCell::new(&mut *edit.admit);
        for role in Role::ALL {
            previous.value.forward[role.index()].visit_group(
                key,
                &mut |member, _| {
                    self.set(
                        role,
                        key,
                        member,
                        0,
                        &mut Editing {
                            registry: edit.registry,
                            admit: &mut **admission.borrow_mut(),
                            step: &mut **work.borrow_mut(),
                            changed: false,
                        },
                    )
                },
                &mut || (**work.borrow_mut())(),
            )?;
            previous.value.reverse[role.index()].visit_group(
                key,
                &mut |owner, _| {
                    self.set(
                        role,
                        owner,
                        key,
                        0,
                        &mut Editing {
                            registry: edit.registry,
                            admit: &mut **admission.borrow_mut(),
                            step: &mut **work.borrow_mut(),
                            changed: false,
                        },
                    )
                },
                &mut || (**work.borrow_mut())(),
            )?;
        }
        if Role::ALL.iter().any(|role| {
            previous.value.has_any(*role, key)
                || previous.value.reverse[role.index()].has_group(key)
        }) {
            edit.changed = true;
        }
        Ok(())
    }
}

fn showing(node: NodeRead<'_>) -> bool {
    matches!(node.kind(), KindRead::Element(element) if element.common_state().html().is_some_and(|state| !state.popover_is_hidden()))
}
fn flagged(node: NodeRead<'_>) -> bool {
    matches!(node.kind(), KindRead::Element(element) if element.fullscreen_flag())
}
fn body_child(
    node: NodeRead<'_>,
    step: &mut dyn FnMut() -> Result<(), DomError>,
) -> Result<bool, DomError> {
    let KindRead::Element(element) = node.kind() else {
        return Ok(false);
    };
    Ok(
        surgeist_html::document_body_role_with(element.name(), |_| step())?
            == surgeist_html::HtmlDocumentBodyRole::Body,
    )
}
fn role_list<'a>(node: NodeRead<'a>, role: Role) -> Option<SequenceView<'a, NodeId, Key>> {
    match role {
        Role::RenderBlocking => node.base_document().map(|base| base.render_blocking),
        Role::AutofocusCandidates => match node.autofocus_document() {
            Some(super::AutofocusRead::Known { candidates, .. }) => Some(candidates),
            _ => None,
        },
        Role::OpenDialogs | Role::TopLayer | Role::PendingTopLayerRemovals => {
            match node.interactive_document() {
                Some(super::InteractiveRead::Known {
                    open_dialogs,
                    top_layer,
                    pending_top_layer_removals,
                }) => Some(match role {
                    Role::OpenDialogs => open_dialogs,
                    Role::TopLayer => top_layer,
                    _ => pending_top_layer_removals,
                }),
                _ => None,
            }
        }
        _ => None,
    }
}

impl Roles {
    pub fn capture(value: &Index, custody: &Custody) -> Self {
        Self {
            base: value.clone(),
            base_custody: custody.clone(),
            current: Version {
                value: value.clone(),
                custody: custody.clone(),
                dirty: Dirty::default(),
            },
        }
    }
    pub fn mark_published(&mut self, value: &Index, custody: &Custody) {
        self.base = value.clone();
        self.base_custody = custody.clone();
        self.current.dirty = Dirty::default();
    }
    pub fn count(&self, role: Role, owner: Key, member: Key) -> usize {
        self.current.value.count(role, owner, member)
    }
    pub fn has_any(&self, role: Role, owner: Key) -> bool {
        self.current.value.has_any(role, owner)
    }
    pub fn visit_owners(
        &self,
        role: Role,
        member: Key,
        visit: &mut impl FnMut(Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.current.value.visit_owners(role, member, visit, step)
    }
    pub fn document_element(
        &self,
        document: Key,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Key>, DomError> {
        self.current.value.document_element(document, step)
    }
    fn prepare(
        &self,
        draft: &Draft,
        cancel: &AtomicBool,
        apply: impl FnOnce(&mut Version, &mut Editing<'_>) -> Result<(), DomError>,
    ) -> Result<Option<Plan>, DomError> {
        let mut next = self.current.clone();
        let mut admit = |usage| draft.admit_custody_usage(usage, cancel);
        let mut step = || draft.source_step(cancel);
        let mut edit = Editing {
            registry: draft.custody_registry(),
            admit: &mut admit,
            step: &mut step,
            changed: false,
        };
        apply(&mut next, &mut edit)?;
        Ok(edit.changed.then_some(Plan { next }))
    }
    pub fn prepare_relation(
        &self,
        draft: &Draft,
        change: RelationChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<Plan>, DomError> {
        let role = Role::relation(change.field);
        if role.is_none() && !matches!(change.field, RelationField::Children) {
            return Ok(None);
        }
        self.prepare(draft, cancel, |next, edit| {
            let owner = draft.live_key(change.owner);
            for (add, values) in [(false, None), (true, Some(change.added))] {
                let count = values.map_or(change.removed, <[Key]>::len);
                for offset in 0..count {
                    (edit.step)()?;
                    let key = draft.live_key(match values {
                        Some(values) => values[offset],
                        None => change
                            .before
                            .get(change.index + offset)
                            .ok_or(DomError::IndexSize)?,
                    });
                    let view = if add { order.after } else { order.before };
                    let node = view.node(key)?;
                    if let Some(role) = role {
                        next.raw_change(role, owner, key, add, node, edit)?;
                    } else {
                        if body_child(node, edit.step)? {
                            next.change(Role::BodyChild, owner, key, add, edit)?;
                        }
                        if matches!(view.node(owner)?.kind(), KindRead::Document(_))
                            && matches!(node.kind(), KindRead::Element(_))
                        {
                            next.change(Role::DocumentElement, owner, key, add, edit)?;
                        }
                    }
                }
            }
            Ok(())
        })
    }
    pub fn prepare_node(
        &self,
        draft: &Draft,
        change: NodeChange<'_>,
        order: OrderPair<'_>,
        cancel: &AtomicBool,
    ) -> Result<Option<Plan>, DomError> {
        self.prepare(draft, cancel, |next, edit| {
            let key = draft.live_key(change.key);
            if change.after.is_none() {
                return next.remove_key(key, edit);
            }
            let before = change.before;
            let after = change.after.expect("non-retired candidate");
            for role in [
                Role::RenderBlocking,
                Role::AutofocusCandidates,
                Role::OpenDialogs,
                Role::TopLayer,
                Role::PendingTopLayerRemovals,
            ] {
                let old = before.and_then(|node| role_list(node, role));
                let new = role_list(after, role);
                // Field guards carry empty sequence sentinels. Their backing is
                // edited by prepare_relation; only availability changes here.
                if old.is_some() == new.is_some() {
                    continue;
                }
                for (add, list) in [(false, old), (true, new)] {
                    if let Some(list) = list {
                        for member in list.iter() {
                            (edit.step)()?;
                            let member = draft.live_key(member);
                            let node = if add {
                                order.after.node(member)?
                            } else {
                                order.before.node(member)?
                            };
                            next.raw_change(role, key, member, add, node, edit)?;
                        }
                    }
                }
            }
            let old_body = match before {
                Some(node) => body_child(node, edit.step)?,
                None => false,
            };
            let new_body = body_child(after, edit.step)?;
            let old_element =
                before.is_some_and(|node| matches!(node.kind(), KindRead::Element(_)));
            let new_element = matches!(after.kind(), KindRead::Element(_));
            for (role, old, new) in [
                (Role::BodyChild, old_body, new_body),
                (Role::DocumentElement, old_element, new_element),
            ] {
                if old == new {
                    continue;
                }
                if old {
                    let previous = next.value.clone();
                    let work = std::cell::RefCell::new(&mut *edit.step);
                    let admission = std::cell::RefCell::new(&mut *edit.admit);
                    let mut changed = false;
                    previous.visit_owners(
                        role,
                        key,
                        &mut |owner, _| {
                            next.set(
                                role,
                                owner,
                                key,
                                0,
                                &mut Editing {
                                    registry: edit.registry,
                                    admit: &mut **admission.borrow_mut(),
                                    step: &mut **work.borrow_mut(),
                                    changed: false,
                                },
                            )?;
                            changed = true;
                            Ok(())
                        },
                        &mut || (**work.borrow_mut())(),
                    )?;
                    edit.changed |= changed;
                }
                if new && let Some(position) = order.after.position(key) {
                    let parent = draft.live_key(position.parent);
                    if role != Role::DocumentElement
                        || matches!(order.after.node(parent)?.kind(), KindRead::Document(_))
                    {
                        next.set(role, parent, key, 1, edit)?;
                    }
                }
            }
            for (role, old, new) in [
                (
                    Role::ShowingPopover,
                    before.is_some_and(showing),
                    showing(after),
                ),
                (
                    Role::FullscreenFlagged,
                    before.is_some_and(flagged),
                    flagged(after),
                ),
            ] {
                if old == new {
                    continue;
                }
                // Actual raw owners may differ from node.document during
                // adoption. Reverse role occurrences are the exact authority.
                let previous = next.value.clone();
                let work = std::cell::RefCell::new(&mut *edit.step);
                let admission = std::cell::RefCell::new(&mut *edit.admit);
                let mut changed = false;
                previous.visit_owners(
                    Role::TopLayer,
                    key,
                    &mut |owner, count| {
                        next.set(
                            role,
                            owner,
                            key,
                            if new { count } else { 0 },
                            &mut Editing {
                                registry: edit.registry,
                                admit: &mut **admission.borrow_mut(),
                                step: &mut **work.borrow_mut(),
                                changed: false,
                            },
                        )?;
                        changed = true;
                        Ok(())
                    },
                    &mut || (**work.borrow_mut())(),
                )?;
                edit.changed |= changed;
            }
            Ok(())
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
    ) -> Result<Option<Plan>, DomError> {
        if was_private == is_private {
            return Ok(None);
        }
        self.prepare(draft, cancel, |next, edit| {
            let key = draft.live_key(key);
            let previous = next.value.clone();
            let work = std::cell::RefCell::new(&mut *edit.step);
            let admission = std::cell::RefCell::new(&mut *edit.admit);
            let mut changed = false;
            for role in Role::ALL {
                previous.forward[role.index()].visit_group(
                    key,
                    &mut |member, _| {
                        next.mark(
                            role,
                            key,
                            member,
                            &mut Editing {
                                registry: edit.registry,
                                admit: &mut **admission.borrow_mut(),
                                step: &mut **work.borrow_mut(),
                                changed: false,
                            },
                        )?;
                        changed = true;
                        Ok(())
                    },
                    &mut || (**work.borrow_mut())(),
                )?;
                previous.visit_owners(
                    role,
                    key,
                    &mut |owner, _| {
                        next.mark(
                            role,
                            owner,
                            key,
                            &mut Editing {
                                registry: edit.registry,
                                admit: &mut **admission.borrow_mut(),
                                step: &mut **work.borrow_mut(),
                                changed: false,
                            },
                        )?;
                        changed = true;
                        Ok(())
                    },
                    &mut || (**work.borrow_mut())(),
                )?;
            }
            edit.changed |= changed;
            Ok(())
        })
    }
    pub fn finish(
        &self,
        draft: &Draft,
        mapper: &impl Fn(Key) -> Option<NodeId>,
        _order: OrderView<'_>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        let mut output = Finished {
            value: self.base.clone(),
            custody: self.base_custody.clone(),
        };
        let work = std::cell::RefCell::new(|| draft.source_step(cancel));
        let admission = std::cell::RefCell::new(|usage| draft.admit_custody_usage(usage, cancel));
        for role in Role::ALL {
            self.current.dirty.value[role.index()].visit(
                &mut |owner, member, _| {
                    let old = draft.published_id(owner).zip(draft.published_id(member));
                    let new = mapper(owner).zip(mapper(member));
                    let mut edit = Editing {
                        registry: draft.custody_registry(),
                        admit: &mut *admission.borrow_mut(),
                        step: &mut *work.borrow_mut(),
                        changed: false,
                    };
                    if old != new
                        && let Some((owner, member)) = old
                    {
                        output.value.set(
                            &mut output.custody,
                            role,
                            Key::Existing(owner),
                            Key::Existing(member),
                            0,
                            &mut edit,
                        )?;
                    }
                    if let Some((mapped_owner, mapped_member)) = new {
                        output.value.set(
                            &mut output.custody,
                            role,
                            Key::Existing(mapped_owner),
                            Key::Existing(mapped_member),
                            self.current.value.count(role, owner, member),
                            &mut edit,
                        )?;
                    }
                    Ok(())
                },
                &mut || (*work.borrow_mut())(),
            )?;
        }
        Ok(output)
    }
}

#[cfg(test)]
impl Index {
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let mut graph = bank::PhysicalGraph::default();
        for role in Role::ALL {
            graph.merge(&self.forward[role.index()].inspect_graph()?)?;
            graph.merge(&self.reverse[role.index()].inspect_graph()?)?;
        }
        Ok(graph)
    }
    pub fn inspect_relations(&self) -> Vec<(Role, Key, Key, usize)> {
        let mut values = Vec::new();
        for role in Role::ALL {
            values.extend(
                self.forward[role.index()]
                    .inspect_relations()
                    .into_iter()
                    .map(|(owner, member, count)| (role, owner, member, count)),
            );
        }
        values
    }
}

#[cfg(test)]
mod tests;
