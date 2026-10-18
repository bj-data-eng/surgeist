//! Private counted relations between exact store-qualified node keys.
//! Both dimensions preserve Existing generations and the Created namespace.
//! Semantic roots carry no leases; nested dependencies retain paired custody.
//! Each consuming index belongs to one DOM owner; exact leaf equality rejects
//! foreign aliases rather than following matching slots from another store.
use super::super::{DomError, Key, bank, custody};
use std::mem::size_of;

const PART_BITS: usize = usize::BITS as usize - 1;
const SLOT_PARTS: usize = (usize::BITS as usize).div_ceil(PART_BITS);
const GENERATION_PARTS: usize = 64_usize.div_ceil(PART_BITS);
const PARTS: usize = 1 + SLOT_PARTS + GENERATION_PARTS;
#[cfg(test)]
pub(super) const KEY_PARTS: usize = PARTS;

fn parts(key: Key) -> [usize; PARTS] {
    let (tag, slot, generation) = match key {
        Key::Existing(id) => (0, id.slot, id.generation),
        Key::Created(index) => (1, index, 0),
    };
    let mask = (1_usize << PART_BITS) - 1;
    let mut parts = [0; PARTS];
    parts[0] = tag;
    for index in 0..SLOT_PARTS {
        parts[1 + index] = (slot >> ((SLOT_PARTS - index - 1) * PART_BITS)) & mask;
    }
    for index in 0..GENERATION_PARTS {
        parts[1 + SLOT_PARTS + index] =
            ((generation >> ((GENERATION_PARTS - index - 1) * PART_BITS)) as usize) & mask;
    }
    parts
}

#[derive(Clone, Default)]
pub(in crate::store) struct Index {
    entries: bank::Bank<Entry>,
    usage: custody::Usage,
    index: bank::IndexCharge,
    records: usize,
}
enum Entry {
    Branch(Index),
    Leaf { key: Key, value: Value },
}
enum Value {
    Members(Index),
    Count(usize),
}
impl Entry {
    fn nested(&self) -> Option<&Index> {
        match self {
            Self::Branch(child)
            | Self::Leaf {
                value: Value::Members(child),
                ..
            } => Some(child),
            Self::Leaf {
                value: Value::Count(_),
                ..
            } => None,
        }
    }
}

fn replace_index(
    total: bank::IndexCharge,
    old: bank::IndexCharge,
    new: bank::IndexCharge,
) -> Result<bank::IndexCharge, DomError> {
    Ok(bank::IndexCharge {
        bytes: total
            .bytes
            .checked_sub(old.bytes)
            .and_then(|n| n.checked_add(new.bytes))
            .ok_or(DomError::RepresentationExhausted)?,
        pages: total
            .pages
            .checked_sub(old.pages)
            .and_then(|n| n.checked_add(new.pages))
            .ok_or(DomError::RepresentationExhausted)?,
    })
}

/// Each branch record's admitted dependency owns its nested custody. The
/// semantic bank retains no dependency tokens, including in external snapshots.
#[derive(Clone, Default)]
pub(in crate::store) struct Custody {
    entries: bank::BankCustody<Entry>,
}

impl Index {
    fn entry(&self, key: Key) -> Option<(Key, &Value)> {
        let mut map = self;
        for index in parts(key) {
            match map.entries.get(index)? {
                Entry::Branch(child) => map = child,
                Entry::Leaf { key: actual, value } => return Some((*actual, value)),
            }
        }
        None
    }
    fn get(&self, key: Key) -> Option<&Value> {
        self.entry(key)
            .and_then(|(actual, value)| (actual == key).then_some(value))
    }
    fn members(&self, key: Key) -> Option<&Index> {
        match self.get(key)? {
            Value::Members(members) => Some(members),
            Value::Count(_) => unreachable!("outer map contains member maps"),
        }
    }
    fn member_count(&self, key: Key) -> usize {
        match self.get(key) {
            Some(Value::Count(count)) => *count,
            None => 0,
            Some(Value::Members(_)) => unreachable!("member map contains counts"),
        }
    }
    fn members_custody(&self, custody: &Custody, key: Key) -> Option<bank::Retained<Custody>> {
        let mut map = self;
        let mut child: Option<bank::Retained<Custody>> = None;
        for index in parts(key) {
            let control = child.as_deref().unwrap_or(custody);
            let record = map.entries.controlled_record(&control.entries, index)?;
            let next = record.dependency::<Custody>();
            match map.entries.get(index)? {
                Entry::Branch(next_map) => map = next_map,
                Entry::Leaf { key: actual, .. } => {
                    return (*actual == key).then_some(next).flatten();
                }
            }
            child = next;
        }
        None
    }
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Custody, DomError> {
        let admission = std::cell::RefCell::new(admit);
        let entries = self.entries.adopt_controlled(
            registry,
            |entry| {
                step()?;
                let usage = custody::Usage::new(size_of::<Entry>(), 0);
                match entry {
                    Entry::Branch(child)
                    | Entry::Leaf {
                        value: Value::Members(child),
                        ..
                    } => {
                        let control = child.adopt_custody(
                            registry,
                            &mut |usage| (*admission.borrow_mut())(usage),
                            step,
                        )?;
                        let dependency =
                            bank::Dependency::new(control, registry.ledger(), |usage| {
                                (*admission.borrow_mut())(usage)
                            })?;
                        Ok(bank::RecordDescription::with_dependency(usage, dependency))
                    }
                    Entry::Leaf {
                        value: Value::Count(_),
                        ..
                    } => Ok(usage.into()),
                }
            },
            |usage| (*admission.borrow_mut())(usage),
        )?;
        Ok(Custody { entries })
    }
    fn visit_values(
        &self,
        visit: &mut impl FnMut(Key, &Value) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        for (_, entry) in self.entries.iter() {
            step()?;
            match entry {
                Entry::Branch(child) => child.visit_values(visit, step)?,
                Entry::Leaf { key, value } => visit(*key, value)?,
            }
        }
        Ok(())
    }
}

struct Editing<'a> {
    registry: &'a bank::CustodyRegistry,
    admit: &'a mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
    step: &'a mut dyn FnMut() -> Result<(), DomError>,
}

struct Replacement {
    entry: Option<Entry>,
    custody: Option<Custody>,
}

/// Each key part is at most usize::MAX/2, preserving Bank's checked extent.
/// Only the selected bounded path and its custody mirror are reconstructed.
fn replace(
    map: &Index,
    custody: &Custody,
    key: Key,
    part: usize,
    replacement: Replacement,
    edit: &mut Editing<'_>,
) -> Result<Changed, DomError> {
    (edit.step)()?;
    let index = parts(key)[part];
    let mut replacement = if part + 1 == PARTS {
        if let Some(Entry::Leaf { key: actual, .. }) = map.entries.get(index)
            && *actual != key
        {
            return Err(DomError::ForeignOwner);
        }
        replacement
    } else {
        let empty = Index::default();
        let empty_custody = Custody::default();
        let child = match map.entries.get(index) {
            Some(Entry::Branch(child)) => child,
            None => &empty,
            Some(Entry::Leaf { .. }) => unreachable!("fixed key prefix contains branches"),
        };
        let dependency = map
            .entries
            .controlled_record(&custody.entries, index)
            .and_then(|record| record.dependency::<Custody>());
        let control = dependency.as_deref().unwrap_or(&empty_custody);
        let next = replace(child, control, key, part + 1, replacement, edit)?;
        if next.value.is_empty() {
            Replacement {
                entry: None,
                custody: None,
            }
        } else {
            Replacement {
                entry: Some(Entry::Branch(next.value)),
                custody: Some(next.custody),
            }
        }
    };
    let previous_nested = map.entries.get(index).and_then(Entry::nested);
    let next_nested = replacement.entry.as_ref().and_then(Entry::nested);
    let nested_index = replace_index(
        replace_index(
            map.index,
            map.entries.index_charge(),
            bank::IndexCharge::default(),
        )?,
        previous_nested.map_or(bank::IndexCharge::default(), |child| child.index),
        next_nested.map_or(bank::IndexCharge::default(), |child| child.index),
    )?;
    let nested_records = map
        .records
        .checked_sub(map.entries.record_count())
        .and_then(|n| n.checked_sub(previous_nested.map_or(0, |child| child.records)))
        .and_then(|n| n.checked_add(next_nested.map_or(0, |child| child.records)))
        .ok_or(DomError::RepresentationExhausted)?;
    let entries = if let Some(entry) = replacement.entry.as_ref() {
        let nested = match entry {
            Entry::Branch(child)
            | Entry::Leaf {
                value: Value::Members(child),
                ..
            } => child.usage,
            Entry::Leaf {
                value: Value::Count(_),
                ..
            } => custody::Usage::ZERO,
        };
        let payload = custody::Usage::new(size_of::<Entry>(), 0).checked_add(nested)?;
        let usage = custody::Usage::new(size_of::<Entry>(), 0);
        let description = if let Some(child) = replacement.custody.as_ref() {
            // Keep the original paired control beside the candidate semantic
            // entry until dependency admission has succeeded.
            let dependency =
                bank::Dependency::new(child.clone(), edit.registry.ledger(), &mut *edit.admit)?;
            bank::RecordDescription::with_dependency(usage, dependency)
        } else {
            usage.into()
        };
        let record = bank::ControlledRecord::new(
            replacement.entry.take().expect("prepared entry"),
            description,
            edit.registry,
            &mut *edit.admit,
        )?;
        map.entries.set_controlled(
            &custody.entries,
            index,
            record,
            false,
            bank::Totals {
                bytes: payload.bytes(),
                units: payload.units(),
                occurrences: 1,
            },
            edit.registry,
            &mut *edit.admit,
        )?
    } else {
        let removed = map.entries.remove_controlled(
            &custody.entries,
            index,
            edit.registry,
            &mut *edit.admit,
        )?;
        let entries = (removed.0, removed.1);
        drop(removed.2);
        entries
    };
    let usage = custody::Usage::new(entries.0.index_charge().bytes, 0).checked_add(
        custody::Usage::new(entries.0.totals().bytes, entries.0.totals().units),
    )?;
    let index = replace_index(
        nested_index,
        bank::IndexCharge::default(),
        entries.0.index_charge(),
    )?;
    let records = nested_records
        .checked_add(entries.0.record_count())
        .ok_or(DomError::RepresentationExhausted)?;
    Ok(Changed {
        value: Index {
            entries: entries.0,
            usage,
            index,
            records,
        },
        custody: Custody { entries: entries.1 },
    })
}

#[derive(Clone, Copy)]
pub(in crate::store) enum Delta {
    Add(usize),
    Remove(usize),
}
/// One atomic count change between the qualified group and member identities.
#[derive(Clone, Copy)]
pub(in crate::store) struct Change {
    pub group: Key,
    pub member: Key,
    pub delta: Delta,
}
impl Delta {
    fn apply(self, previous: usize) -> Result<usize, DomError> {
        match self {
            Self::Add(count) => previous.checked_add(count),
            Self::Remove(count) => previous.checked_sub(count),
        }
        .ok_or(DomError::RepresentationExhausted)
    }
}

fn adjust(
    map: &Index,
    custody: &Custody,
    group: Key,
    member: Key,
    delta: Delta,
    edit: &mut Editing<'_>,
) -> Result<Changed, DomError> {
    if map.entry(group).is_some_and(|(actual, _)| actual != group) {
        return Err(DomError::ForeignOwner);
    }
    let empty = Index::default();
    let empty_custody = Custody::default();
    let members = map.members(group).unwrap_or(&empty);
    let members_custody = map.members_custody(custody, group);
    let control = members_custody.as_deref().unwrap_or(&empty_custody);
    let count = delta.apply(members.member_count(member))?;
    let members = replace(
        members,
        control,
        member,
        0,
        Replacement {
            entry: (count != 0).then_some(Entry::Leaf {
                key: member,
                value: Value::Count(count),
            }),
            custody: None,
        },
        edit,
    )?;
    let nonempty = !members.value.is_empty();
    replace(
        map,
        custody,
        group,
        0,
        Replacement {
            entry: nonempty.then_some(Entry::Leaf {
                key: group,
                value: Value::Members(members.value),
            }),
            custody: nonempty.then_some(members.custody),
        },
        edit,
    )
}

/// A prepared replacement keeps semantic storage before its matching control.
pub(in crate::store) struct Changed {
    pub value: Index,
    pub custody: Custody,
}
impl std::fmt::Debug for Index {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        out.debug_struct("CountedRelations")
            .field("retained_usage", &self.usage)
            .finish_non_exhaustive()
    }
}
impl Index {
    pub fn retained_usage(&self) -> custody::Usage {
        self.usage
    }
    #[cfg(test)]
    pub fn index_charge(&self) -> bank::IndexCharge {
        self.index
    }
    #[cfg(test)]
    pub fn totals(&self) -> bank::Totals {
        bank::Totals {
            bytes: self
                .usage
                .bytes()
                .checked_sub(self.index.bytes)
                .expect("checked counted-relation payload"),
            units: self.usage.units(),
            occurrences: self.records,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.entries.record_count() == 0
    }
    pub fn count(&self, group: Key, member: Key) -> usize {
        self.members(group)
            .map_or(0, |members| members.member_count(member))
    }
    pub fn has_group(&self, group: Key) -> bool {
        self.members(group)
            .is_some_and(|members| !members.is_empty())
    }
    #[cfg(test)]
    pub(super) fn count_record_extent() -> custody::Usage {
        custody::Usage::new(size_of::<Entry>(), 0)
            .checked_add(custody::SharedDebit::administration_usage())
            .expect("actual first count record and its out-of-line debit")
    }
    pub fn change(
        &self,
        custody: &Custody,
        change: Change,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Changed, DomError> {
        adjust(
            self,
            custody,
            change.group,
            change.member,
            change.delta,
            &mut Editing {
                registry,
                admit: &mut admit,
                step: &mut step,
            },
        )
    }
    pub fn visit_group(
        &self,
        group: Key,
        visit: &mut impl FnMut(Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if let Some(members) = self.members(group) {
            members.visit_values(
                &mut |key, value| {
                    let Value::Count(count) = value else {
                        unreachable!("member count leaf")
                    };
                    visit(key, *count)
                },
                step,
            )?;
        }
        Ok(())
    }
    pub fn visit(
        &self,
        visit: &mut impl FnMut(Key, Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        Self::visit_groups(self, visit, step)
    }
    /// Select the Created namespace without visiting canonical groups.
    pub fn visit_created(
        &self,
        visit: &mut impl FnMut(Key, Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        if let Some(Entry::Branch(created)) = self.entries.get(1) {
            Self::visit_groups(created, visit, step)?;
        }
        Ok(())
    }
    fn visit_groups(
        groups: &Self,
        visit: &mut impl FnMut(Key, Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let steps = std::cell::RefCell::new(step);
        groups.visit_values(
            &mut |group, value| {
                let Value::Members(members) = value else {
                    unreachable!("group member map")
                };
                members.visit_values(
                    &mut |member, value| {
                        let Value::Count(count) = value else {
                            unreachable!("member count leaf")
                        };
                        visit(group, member, *count)
                    },
                    &mut || (*steps.borrow_mut())(),
                )
            },
            &mut || (*steps.borrow_mut())(),
        )
    }
}
#[cfg(test)]
impl Index {
    pub fn shares_root(&self, other: &Self) -> bool {
        self.entries.shares_root(&other.entries)
    }

    pub fn inspect_relations(&self) -> Vec<(Key, Key, usize)> {
        fn members(map: &Index, group: Key, out: &mut Vec<(Key, Key, usize)>) {
            for (_, entry) in map.entries.iter() {
                match entry {
                    Entry::Branch(child) => members(child, group, out),
                    Entry::Leaf {
                        key,
                        value: Value::Count(count),
                    } => out.push((group, *key, *count)),
                    Entry::Leaf {
                        value: Value::Members(_),
                        ..
                    } => unreachable!("member count leaves"),
                }
            }
        }
        fn groups(map: &Index, out: &mut Vec<(Key, Key, usize)>) {
            for (_, entry) in map.entries.iter() {
                match entry {
                    Entry::Branch(child) => groups(child, out),
                    Entry::Leaf {
                        key,
                        value: Value::Members(child),
                    } => members(child, *key, out),
                    Entry::Leaf {
                        value: Value::Count(_),
                        ..
                    } => unreachable!("group member maps"),
                }
            }
        }
        let mut out = Vec::new();
        groups(self, &mut out);
        out
    }
}
#[cfg(test)]
impl Index {
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let mut graph = self.entries.inspect_graph(|_| {
            Ok(bank::Totals {
                bytes: size_of::<Entry>(),
                units: 0,
                occurrences: 1,
            })
        })?;
        for (_, entry) in self.entries.iter() {
            match entry {
                Entry::Branch(child)
                | Entry::Leaf {
                    value: Value::Members(child),
                    ..
                } => {
                    graph.merge(&child.inspect_graph()?)?;
                }
                Entry::Leaf {
                    value: Value::Count(_),
                    ..
                } => {}
            }
        }
        Ok(graph)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomOwnerId, NodeId};

    fn key(owner: DomOwnerId, slot: usize, generation: u64) -> Key {
        Key::Existing(NodeId {
            owner,
            slot,
            generation,
        })
    }
    fn registry() -> (custody::Ledger, bank::CustodyRegistry) {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        (ledger, registry)
    }
    fn change(
        pair: &mut Changed,
        group: Key,
        member: Key,
        delta: Delta,
        registry: &bank::CustodyRegistry,
    ) {
        *pair = pair
            .value
            .change(
                &pair.custody,
                Change {
                    group,
                    member,
                    delta,
                },
                registry,
                |_| Ok(()),
                || Ok(()),
            )
            .unwrap();
    }

    #[test]
    fn both_dimensions_keep_generations_created_tags_and_multiplicities() {
        let (_, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let group = key(owner, 7, 1);
        let reused_group = key(owner, 7, 2);
        let member = key(owner, 9, 1);
        let reused_member = key(owner, 9, 2);
        let mut pair = Changed {
            value: Index::default(),
            custody: Custody::default(),
        };
        for (group, member, count) in [
            (group, member, 2),
            (group, reused_member, 3),
            (group, Key::Created(9), 4),
            (reused_group, member, 5),
            (Key::Created(7), member, 6),
        ] {
            change(&mut pair, group, member, Delta::Add(count), &registry);
        }
        let mut selected = Vec::new();
        pair.value
            .visit_group(
                group,
                &mut |key, count| {
                    selected.push((key, count));
                    Ok(())
                },
                &mut || Ok(()),
            )
            .unwrap();
        assert_eq!(selected.len(), 3);
        assert!(selected.contains(&(member, 2)));
        assert!(selected.contains(&(reused_member, 3)));
        assert!(selected.contains(&(Key::Created(9), 4)));
        change(&mut pair, group, member, Delta::Remove(1), &registry);
        assert_eq!(pair.value.count(group, member), 1);
        assert_eq!(pair.value.count(group, reused_member), 3);
        assert_eq!(pair.value.count(reused_group, member), 5);
        assert_eq!(pair.value.count(Key::Created(7), member), 6);
        let foreign = DomOwnerId::allocate().unwrap();
        assert_eq!(pair.value.count(key(foreign, 7, 1), member), 0);
        assert_eq!(pair.value.count(group, key(foreign, 9, 1)), 0);
        assert!(matches!(
            pair.value.change(
                &pair.custody,
                Change {
                    group: key(foreign, 7, 1),
                    member,
                    delta: Delta::Add(1)
                },
                &registry,
                |_| Ok(()),
                || Ok(())
            ),
            Err(DomError::ForeignOwner)
        ));
        assert!(matches!(
            pair.value.change(
                &pair.custody,
                Change {
                    group,
                    member: key(foreign, 9, 1),
                    delta: Delta::Add(1)
                },
                &registry,
                |_| Ok(()),
                || Ok(())
            ),
            Err(DomError::ForeignOwner)
        ));
        let graph = pair.value.inspect_graph().unwrap();
        assert_eq!(
            pair.value.retained_usage(),
            custody::Usage::new(graph.index_charge().bytes + graph.record_totals().bytes, 0)
        );
        assert_eq!(pair.value.index_charge(), graph.index_charge());
        assert_eq!(pair.value.totals(), graph.record_totals());
    }

    #[test]
    fn selected_group_visits_no_unrelated_groups_and_refusal_preserves_the_pair() {
        let (ledger, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let group = key(owner, 7, 1);
        let member = key(owner, 9, 1);
        let mut pair = Changed {
            value: Index::default(),
            custody: Custody::default(),
        };
        change(&mut pair, group, member, Delta::Add(2), &registry);
        let visits = |index: &Index| {
            let mut calls = 0;
            index
                .visit_group(
                    group,
                    &mut |key, count| {
                        assert_eq!((key, count), (member, 2));
                        Ok(())
                    },
                    &mut || {
                        calls += 1;
                        Ok(())
                    },
                )
                .unwrap();
            calls
        };
        let before = visits(&pair.value);
        for slot in 20..84 {
            change(
                &mut pair,
                key(owner, slot, 1),
                member,
                Delta::Add(1),
                &registry,
            );
        }
        assert_eq!(visits(&pair.value), before);
        registry.reconcile_releases();
        let before = ledger.total();
        let mut calls = 0;
        let refused = pair.value.change(
            &pair.custody,
            Change {
                group,
                member: key(owner, 10, 1),
                delta: Delta::Add(1),
            },
            &registry,
            |_| {
                calls += 1;
                if calls == 8 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
            || Ok(()),
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        assert_eq!(calls, 8);
        assert_eq!(pair.value.count(group, member), 2);
        assert_eq!(pair.value.count(group, key(owner, 10, 1)), 0);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), before);
    }

    #[test]
    fn historical_adoption_recovers_nested_counts_and_active_capture_prunes() {
        let (ledger, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let group = key(owner, 7, 1);
        let member = key(owner, 9, u64::MAX);
        let mut pair = Changed {
            value: Index::default(),
            custody: Custody::default(),
        };
        change(&mut pair, group, member, Delta::Add(2), &registry);
        let external = pair.value.clone();
        let mut descriptors = 0;
        let alias = external
            .adopt_custody(&registry, &mut |_| Ok(()), &mut || {
                descriptors += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(descriptors, 0);
        drop(alias);
        drop(pair);
        registry.reconcile_releases();
        let baseline = ledger.total();
        let adopted = external
            .adopt_custody(&registry, &mut |_| Ok(()), &mut || {
                descriptors += 1;
                Ok(())
            })
            .unwrap();
        assert!(descriptors > 0);
        assert!(ledger.total().bytes() > baseline.bytes());
        assert_eq!(external.count(group, member), 2);
        drop(adopted);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }

    #[test]
    fn refusals_release_partial_nested_replacements_and_preserve_original_custody() {
        let (ledger, registry) = registry();
        let owner = DomOwnerId::allocate().unwrap();
        let group = key(owner, 7, 1);
        let member = key(owner, 9, 1);
        let added = key(owner, 10, u64::MAX);
        let mut pair = Changed {
            value: Index::default(),
            custody: Custody::default(),
        };
        change(&mut pair, group, member, Delta::Add(2), &registry);
        registry.reconcile_releases();
        let before = ledger.total();
        for refused_step in 1..=2 * PARTS {
            let mut steps = 0;
            let refused = pair.value.change(
                &pair.custody,
                Change {
                    group,
                    member: added,
                    delta: Delta::Add(3),
                },
                &registry,
                |_| Ok(()),
                || {
                    steps += 1;
                    if steps == refused_step {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                },
            );
            assert!(matches!(refused, Err(DomError::Cancelled)));
            registry.reconcile_releases();
            assert_eq!(ledger.total(), before);
            assert_eq!(pair.value.count(group, member), 2);
            assert_eq!(pair.value.count(group, added), 0);
        }
        let complete = pair
            .value
            .change(
                &pair.custody,
                Change {
                    group,
                    member: added,
                    delta: Delta::Add(3),
                },
                &registry,
                |_| Ok(()),
                || Ok(()),
            )
            .unwrap();
        assert_eq!(complete.value.count(group, member), 2);
        assert_eq!(complete.value.count(group, added), 3);
        drop(complete);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), before);
        for refused_prefix in [PARTS, 2 * PARTS] {
            let steps = std::cell::Cell::new(0);
            let refused = pair.value.change(
                &pair.custody,
                Change {
                    group,
                    member: added,
                    delta: Delta::Add(3),
                },
                &registry,
                |_| {
                    if steps.get() >= refused_prefix {
                        Err(DomError::Cancelled)
                    } else {
                        Ok(())
                    }
                },
                || {
                    steps.set(steps.get() + 1);
                    Ok(())
                },
            );
            assert!(matches!(refused, Err(DomError::Cancelled)));
            registry.reconcile_releases();
            assert_eq!(ledger.total(), before);
            assert_eq!(pair.value.count(group, member), 2);
            assert_eq!(pair.value.count(group, added), 0);
        }
    }
}
