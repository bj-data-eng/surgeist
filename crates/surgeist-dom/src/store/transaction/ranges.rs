//! Sparse Range records and exact derived endpoint/association indexes.
//!
//! Index nodes use private pool indices, never semantic NodeIds. Keeping the
//! fixed-fanout membership nodes in an ordinary paired Bank lets snapshots hold
//! lease-free derived state while live owners retain only changed index paths.

use super::super::*;
use super::{Counts, StateCustody};
use crate::range::{Record as RangeRecord, Slot as RangeSlot};
use crate::{DomActiveRangeState, DomRangeId};

const BITS: u32 = 4;
const WIDTH: usize = 1 << BITS;
const MASK: usize = WIDTH - 1;
const SHIFT: u32 = usize::BITS - BITS;

#[cfg(test)]
fn allocated(bytes: usize) {
    if scale_metrics::ENABLED {
        scale_metrics::record(scale_metrics::Event::StructuralAllocation { bytes });
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Member {
    Range(DomRangeId),
    Document(Key),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[expect(
    clippy::large_enum_variant,
    reason = "Fixed-fanout Copy nodes use one admitted bank record; boxing adds a separate allocation and receipt"
)]
enum IndexNode {
    Branch([Option<usize>; WIDTH]),
    Leaf([Option<Member>; WIDTH]),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Container {
    key: Key,
    root: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Associations {
    id: DomRangeId,
    existing: Option<usize>,
    created: Option<usize>,
}

#[derive(Clone, Copy)]
struct IndexPath {
    root: Option<usize>,
    shift: u32,
}

/// Canonical derived state. These values contain no accounting authority.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(in crate::store) struct RangeDependencies {
    availability: bank::Bank<()>,
    containers: bank::Bank<Container>,
    created_containers: bank::Bank<Container>,
    associations: bank::Bank<Associations>,
    nodes: bank::Bank<IndexNode>,
}

#[derive(Clone, Default)]
pub(in crate::store) struct DependencyCustody {
    availability: bank::BankCustody<()>,
    containers: bank::BankCustody<Container>,
    created_containers: bank::BankCustody<Container>,
    associations: bank::BankCustody<Associations>,
    nodes: bank::BankCustody<IndexNode>,
}

#[cfg(test)]
pub(in crate::store) struct DependencyMemberships {
    pub containers: Vec<(Key, Vec<DomRangeId>)>,
    pub associations: Vec<(DomRangeId, Vec<Key>)>,
}

impl RangeDependencies {
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<DependencyCustody, DomError> {
        fn adopt<T>(
            bank: &bank::Bank<T>,
            registry: &bank::CustodyRegistry,
            admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
            step: &mut impl FnMut() -> Result<(), DomError>,
        ) -> Result<bank::BankCustody<T>, DomError> {
            bank.adopt_controlled(
                registry,
                |_| {
                    step()?;
                    Ok(custody::Usage::new(size_of::<T>(), 0))
                },
                admit,
            )
        }
        let _cleanup = Reconcile(registry.clone());
        Ok(DependencyCustody {
            availability: adopt(&self.availability, registry, admit, step)?,
            containers: adopt(&self.containers, registry, admit, step)?,
            created_containers: adopt(&self.created_containers, registry, admit, step)?,
            associations: adopt(&self.associations, registry, admit, step)?,
            nodes: adopt(&self.nodes, registry, admit, step)?,
        })
    }

    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        fn graph<T>(bank: &bank::Bank<T>) -> Result<bank::PhysicalGraph, DomError> {
            bank.inspect_graph(|_| {
                Ok(bank::Totals {
                    bytes: size_of::<T>(),
                    units: 0,
                    occurrences: 1,
                })
            })
        }
        let mut physical = graph(&self.availability)?;
        physical.merge(&graph(&self.containers)?)?;
        physical.merge(&graph(&self.created_containers)?)?;
        physical.merge(&graph(&self.associations)?)?;
        physical.merge(&graph(&self.nodes)?)?;
        Ok(physical)
    }

    /// Cached quantities under test, separate from the independent graph.
    #[cfg(test)]
    pub fn inspect_summaries(&self) -> [(bank::Totals, bank::IndexCharge); 5] {
        [
            (self.availability.totals(), self.availability.index_charge()),
            (self.containers.totals(), self.containers.index_charge()),
            (
                self.created_containers.totals(),
                self.created_containers.index_charge(),
            ),
            (self.associations.totals(), self.associations.index_charge()),
            (self.nodes.totals(), self.nodes.index_charge()),
        ]
    }

    /// Decode actual derived edges for the independent whole-State oracle.
    /// This deliberately bypasses production participant/association queries.
    #[cfg(test)]
    pub fn inspect_memberships(&self) -> DependencyMemberships {
        fn decode(nodes: &bank::Bank<IndexNode>, root: usize) -> Vec<Member> {
            let mut pending = vec![root];
            let mut seen = std::collections::BTreeSet::new();
            let mut values = Vec::new();
            while let Some(index) = pending.pop() {
                assert!(seen.insert(index), "acyclic derived Range membership tree");
                match nodes.get(index).expect("actual derived membership edge") {
                    IndexNode::Branch(children) => {
                        pending.extend(children.iter().rev().flatten().copied());
                    }
                    IndexNode::Leaf(members) => values.extend(members.iter().flatten().copied()),
                }
            }
            values
        }
        let containers = self
            .containers
            .values()
            .chain(self.created_containers.values())
            .map(|container| {
                let ranges = decode(&self.nodes, container.root)
                    .into_iter()
                    .map(|member| {
                        let Member::Range(id) = member else {
                            panic!("container membership is a Range")
                        };
                        id
                    })
                    .collect();
                (container.key, ranges)
            })
            .collect();
        let associations = self
            .associations
            .values()
            .map(|association| {
                let mut documents = Vec::new();
                for root in [association.existing, association.created]
                    .into_iter()
                    .flatten()
                {
                    documents.extend(decode(&self.nodes, root).into_iter().map(|member| {
                        let Member::Document(key) = member else {
                            panic!("association membership is a Document")
                        };
                        key
                    }));
                }
                (association.id, documents)
            })
            .collect();
        DependencyMemberships {
            containers,
            associations,
        }
    }

    /// Materialized metadata slots and the actual cached allocator answer.
    /// The oracle derives its expected eligibility independently from Ranges.
    #[cfg(test)]
    pub fn inspect_availability(&self) -> (Vec<usize>, Option<usize>) {
        (
            self.availability.iter().map(|(index, ())| index).collect(),
            self.availability.lowest_vacant(),
        )
    }
}

/// The field order applies to errors and discarded candidate/undo versions too.
#[derive(Clone, Default)]
struct Version {
    overlay: bank::Bank<RangeSlot<Key>>,
    dependencies: RangeDependencies,
    active_touched: bank::Bank<DomRangeId>,
    active_documents: bank::Bank<NodeId>,
    active_created_documents: bank::Bank<usize>,
    overlay_custody: bank::BankCustody<RangeSlot<Key>>,
    dependency_custody: DependencyCustody,
    active_touched_custody: bank::BankCustody<DomRangeId>,
    active_documents_custody: bank::BankCustody<NodeId>,
    active_created_documents_custody: bank::BankCustody<usize>,
}

pub(in crate::store) struct Ranges {
    base: bank::Bank<RangeSlot<NodeId>>,
    base_dependencies: RangeDependencies,
    current: Version,
    base_custody: bank::BankCustody<RangeSlot<NodeId>>,
    base_dependency_custody: DependencyCustody,
}

pub(in crate::store) struct RangeUndo {
    change: RangeChange,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Journal capacity already admits the inline version; boxing adds an allocation and custody to every inverse"
)]
enum RangeChange {
    State {
        before: Version,
        counts: Counts,
        tickets: usize,
    },
    Tickets {
        before: custody::Held<Vec<DomRangeId>>,
    },
}

impl RangeUndo {
    /// Private backing retained only by this inverse. Shared persistent paths
    /// remain in the ledger union; the inline holder is in journal capacity.
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        match &self.change {
            RangeChange::State { .. } => 0,
            RangeChange::Tickets { before } => before.debit().usage().bytes(),
        }
    }

    pub fn restore(self, draft: &mut Draft) {
        match self.change {
            RangeChange::State {
                before,
                counts,
                tickets,
            } => {
                draft.range_tickets.truncate(tickets);
                draft.ranges.current = before;
                draft.counts = counts;
            }
            RangeChange::Tickets { before } => {
                let (values, debit) = before.into_parts();
                let old = custody::Held::new(
                    std::mem::replace(&mut draft.range_tickets, values),
                    std::mem::replace(&mut draft.range_tickets_charge, debit),
                );
                drop(old);
            }
        }
        draft.custody_registry().reconcile_releases();
    }
}

pub(in crate::store) struct Finished {
    pub ranges: bank::Bank<RangeSlot<NodeId>>,
    pub dependencies: RangeDependencies,
    pub custody: bank::BankCustody<RangeSlot<NodeId>>,
    pub dependency_custody: DependencyCustody,
}

impl Ranges {
    pub fn new(base: &State, custody: &StateCustody) -> Self {
        Self {
            base: base.ranges.clone(),
            base_dependencies: base.range_dependencies.clone(),
            current: Version {
                dependencies: base.range_dependencies.clone(),
                dependency_custody: custody.range_dependencies.clone(),
                ..Version::default()
            },
            base_custody: custody.ranges.clone(),
            base_dependency_custody: custody.range_dependencies.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.base.len().max(self.current.overlay.len())
    }

    pub fn slot(&self, index: usize) -> Option<RangeSlot<Key>> {
        if let Some(slot) = self.current.overlay.get(index) {
            return Some(slot.clone());
        }
        if index >= self.base.len() {
            return None;
        }
        Some(self.base.get(index).map_or_else(vacant, |slot| RangeSlot {
            generation: slot.generation,
            retired: slot.retired,
            record: slot.record.map(|record| record.map(Key::Existing)),
        }))
    }

    pub fn lowest_vacant(&self) -> Option<usize> {
        self.current
            .dependencies
            .availability
            .lowest_vacant()
            .filter(|index| *index < self.len())
    }

    pub fn touched(&self) -> impl Iterator<Item = usize> + '_ {
        self.current.overlay.iter().map(|(index, _)| index)
    }

    pub fn changed_records(&self) -> impl Iterator<Item = (usize, &RangeSlot<Key>)> + '_ {
        self.current.overlay.iter()
    }

    #[cfg(test)]
    pub fn allocation_graph(&self) -> bank::PhysicalGraph {
        self.current
            .dependencies
            .availability
            .inspect_graph(|()| {
                Ok(bank::Totals {
                    bytes: 0,
                    units: 0,
                    occurrences: 1,
                })
            })
            .unwrap()
    }
}

fn vacant<N>() -> RangeSlot<N> {
    RangeSlot {
        generation: 0,
        retired: false,
        record: None,
    }
}
fn reusable<N>(slot: &RangeSlot<N>) -> bool {
    slot.record.is_none() && !slot.retired && slot.generation < u64::MAX
}

struct Pair<T> {
    semantic: bank::Bank<T>,
    custody: bank::BankCustody<T>,
}

impl<T> Drop for Pair<T> {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.semantic));
        drop(std::mem::take(&mut self.custody));
    }
}

struct Reconcile(bank::CustodyRegistry);
impl Drop for Reconcile {
    fn drop(&mut self) {
        self.0.reconcile_releases();
    }
}

fn set<T>(
    bank: &mut bank::Bank<T>,
    custody: &mut bank::BankCustody<T>,
    index: usize,
    value: T,
    reusable: bool,
    registry: &bank::CustodyRegistry,
    admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let bytes = size_of::<T>();
    let value =
        bank::ControlledRecord::new(value, custody::Usage::new(bytes, 0), registry, &mut *admit)?;
    let (next, next_custody) = bank.set_controlled(
        custody,
        index,
        value,
        reusable,
        bank::Totals {
            bytes,
            units: 0,
            occurrences: 1,
        },
        registry,
        &mut *admit,
    )?;
    let old = Pair {
        semantic: std::mem::replace(bank, next),
        custody: std::mem::replace(custody, next_custody),
    };
    drop(old);
    registry.reconcile_releases();
    Ok(())
}

fn remove<T>(
    bank: &mut bank::Bank<T>,
    custody: &mut bank::BankCustody<T>,
    index: usize,
    registry: &bank::CustodyRegistry,
    admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let (next, next_custody, removed) =
        bank.remove_controlled(custody, index, registry, &mut *admit)?;
    let old = Pair {
        semantic: std::mem::replace(bank, next),
        custody: std::mem::replace(custody, next_custody),
    };
    drop(old);
    drop(removed);
    registry.reconcile_releases();
    Ok(())
}

impl Version {
    fn member(
        &mut self,
        path: IndexPath,
        key: usize,
        value: Option<Member>,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<usize>, DomError> {
        let IndexPath { root, shift } = path;
        step()?;
        let previous = root.map(|index| {
            *self
                .dependencies
                .nodes
                .get(index)
                .expect("live derived Range index node")
        });
        let next = if shift == 0 {
            let mut values = match previous {
                Some(IndexNode::Leaf(values)) => values,
                None => [None; WIDTH],
                _ => unreachable!("leaf Range index depth"),
            };
            values[key & MASK] = value;
            values
                .iter()
                .any(Option::is_some)
                .then_some(IndexNode::Leaf(values))
        } else {
            let mut children = match previous {
                Some(IndexNode::Branch(values)) => values,
                None => [None; WIDTH],
                _ => unreachable!("branch Range index depth"),
            };
            let digit = (key >> shift) & MASK;
            children[digit] = self.member(
                IndexPath {
                    root: children[digit],
                    shift: shift - BITS,
                },
                key,
                value,
                registry,
                admit,
                step,
            )?;
            children
                .iter()
                .any(Option::is_some)
                .then_some(IndexNode::Branch(children))
        };
        if previous == next {
            return Ok(root);
        }
        match next {
            Some(next) => {
                let index = root.unwrap_or_else(|| {
                    self.dependencies
                        .nodes
                        .lowest_vacant()
                        .unwrap_or(self.dependencies.nodes.len())
                });
                set(
                    &mut self.dependencies.nodes,
                    &mut self.dependency_custody.nodes,
                    index,
                    next,
                    false,
                    registry,
                    admit,
                )?;
                Ok(Some(index))
            }
            None => {
                if let Some(index) = root {
                    remove(
                        &mut self.dependencies.nodes,
                        &mut self.dependency_custody.nodes,
                        index,
                        registry,
                        admit,
                    )?;
                }
                Ok(None)
            }
        }
    }

    fn container(
        &mut self,
        key: Key,
        id: DomRangeId,
        present: bool,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let (index, created) = match key {
            Key::Existing(id) => (id.slot, false),
            Key::Created(index) => (index, true),
        };
        let bank = if created {
            &self.dependencies.created_containers
        } else {
            &self.dependencies.containers
        };
        let old = bank.get(index).copied();
        if old.is_some_and(|old| old.key != key) {
            return Err(DomError::StaleNode);
        }
        let root = self.member(
            IndexPath {
                root: old.map(|old| old.root),
                shift: SHIFT,
            },
            id.slot,
            present.then_some(Member::Range(id)),
            registry,
            admit,
            step,
        )?;
        let (bank, custody) = if created {
            (
                &mut self.dependencies.created_containers,
                &mut self.dependency_custody.created_containers,
            )
        } else {
            (
                &mut self.dependencies.containers,
                &mut self.dependency_custody.containers,
            )
        };
        match root {
            Some(root) => set(
                bank,
                custody,
                index,
                Container { key, root },
                false,
                registry,
                admit,
            ),
            None => remove(bank, custody, index, registry, admit),
        }
    }

    fn association(
        &mut self,
        document: Key,
        id: DomRangeId,
        present: bool,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let mut association = self
            .dependencies
            .associations
            .get(id.slot)
            .copied()
            .unwrap_or(Associations {
                id,
                existing: None,
                created: None,
            });
        if association.id != id {
            return Err(DomError::StaleRange);
        }
        let (key, root) = match document {
            Key::Existing(document) => (document.slot, &mut association.existing),
            Key::Created(index) => (index, &mut association.created),
        };
        *root = self.member(
            IndexPath {
                root: *root,
                shift: SHIFT,
            },
            key,
            present.then_some(Member::Document(document)),
            registry,
            admit,
            step,
        )?;
        if association.existing.is_none() && association.created.is_none() {
            remove(
                &mut self.dependencies.associations,
                &mut self.dependency_custody.associations,
                id.slot,
                registry,
                admit,
            )?;
        } else {
            set(
                &mut self.dependencies.associations,
                &mut self.dependency_custody.associations,
                id.slot,
                association,
                false,
                registry,
                admit,
            )?;
        }
        set(
            &mut self.active_touched,
            &mut self.active_touched_custody,
            id.slot,
            id,
            false,
            registry,
            admit,
        )
    }
}

fn members(
    nodes: &bank::Bank<IndexNode>,
    root: usize,
    step: &mut impl FnMut() -> Result<(), DomError>,
    visit: &mut impl FnMut(Member) -> Result<(), DomError>,
) -> Result<(), DomError> {
    step()?;
    match nodes.get(root).expect("live derived Range membership root") {
        IndexNode::Leaf(values) => {
            for value in values.iter().flatten() {
                step()?;
                visit(*value)?;
            }
        }
        IndexNode::Branch(children) => {
            for child in children.iter().flatten() {
                members(nodes, *child, step, visit)?;
            }
        }
    }
    Ok(())
}

/// A real temporary participant set, ordered by original Range slot. It is
/// independent of the mutable dependency candidate while maintenance runs.
pub(in crate::store) struct Participants {
    pair: Pair<DomRangeId>,
    registry: bank::CustodyRegistry,
}

impl Participants {
    fn new(registry: &bank::CustodyRegistry) -> Self {
        Self {
            pair: Pair {
                semantic: bank::Bank::default(),
                custody: bank::BankCustody::default(),
            },
            registry: registry.clone(),
        }
    }
    fn insert(
        &mut self,
        id: DomRangeId,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.pair.semantic.get(id.slot) == Some(&id) {
            return Ok(());
        }
        set(
            &mut self.pair.semantic,
            &mut self.pair.custody,
            id.slot,
            id,
            false,
            &self.registry,
            &mut |usage| draft.admit_custody_usage(usage, cancel),
        )
    }
    pub fn iter(&self) -> impl Iterator<Item = DomRangeId> + '_ {
        self.pair.semantic.values().copied()
    }
}

impl Drop for Participants {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.pair.semantic));
        drop(std::mem::take(&mut self.pair.custody));
        self.registry.reconcile_releases();
    }
}

impl Draft {
    pub(in crate::store) fn write_range(
        &mut self,
        index: usize,
        mut slot: RangeSlot<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        slot.record = slot
            .record
            .map(|record| record.map(|key| self.canonical_key(key)));
        let old = self.ranges.slot(index);
        if old.as_ref() == Some(&slot) {
            return Ok(());
        }
        let retain = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut candidate = self.ranges.current.clone();
        let registry = self.custody_registry();
        let mut admit = |usage| self.admit_custody_usage(usage, cancel);
        let mut step = || self.source_step(cancel);
        let same_identity = old
            .as_ref()
            .is_some_and(|old| old.generation == slot.generation);
        let contains = |record: Option<RangeRecord<Key>>, container| {
            record.is_some_and(|record| {
                record.start.container == container || record.end.container == container
            })
        };
        if let Some(old) = &old
            && let Some(record) = old.record
        {
            let id = DomRangeId {
                owner: self.store,
                slot: index,
                generation: old.generation,
            };
            if !same_identity || !contains(slot.record, record.start.container) {
                candidate.container(
                    record.start.container,
                    id,
                    false,
                    registry,
                    &mut admit,
                    &mut step,
                )?;
            }
            if record.end.container != record.start.container
                && (!same_identity || !contains(slot.record, record.end.container))
            {
                candidate.container(
                    record.end.container,
                    id,
                    false,
                    registry,
                    &mut admit,
                    &mut step,
                )?;
            }
        }
        if let Some(record) = slot.record {
            let id = DomRangeId {
                owner: self.store,
                slot: index,
                generation: slot.generation,
            };
            if !same_identity
                || !contains(
                    old.as_ref().and_then(|slot| slot.record),
                    record.start.container,
                )
            {
                candidate.container(
                    record.start.container,
                    id,
                    true,
                    registry,
                    &mut admit,
                    &mut step,
                )?;
            }
            if record.end.container != record.start.container
                && (!same_identity
                    || !contains(
                        old.as_ref().and_then(|slot| slot.record),
                        record.end.container,
                    ))
            {
                candidate.container(
                    record.end.container,
                    id,
                    true,
                    registry,
                    &mut admit,
                    &mut step,
                )?;
            }
        }
        if old
            .as_ref()
            .is_none_or(|old| reusable(old) != reusable(&slot))
        {
            set(
                &mut candidate.dependencies.availability,
                &mut candidate.dependency_custody.availability,
                index,
                (),
                reusable(&slot),
                registry,
                &mut admit,
            )?;
        }
        set(
            &mut candidate.overlay,
            &mut candidate.overlay_custody,
            index,
            slot.clone(),
            false,
            registry,
            &mut admit,
        )?;
        let mut counts = self.counts;
        counts.live_ranges = counts
            .live_ranges
            .checked_sub(usize::from(
                old.as_ref().is_some_and(|slot| slot.record.is_some()),
            ))
            .and_then(|count| count.checked_add(usize::from(slot.record.is_some())))
            .ok_or(DomError::RepresentationExhausted)?;
        let before = std::mem::replace(&mut self.ranges.current, candidate);
        let counts = std::mem::replace(&mut self.counts, counts);
        if retain {
            self.record_range_undo_prepared(RangeUndo {
                change: RangeChange::State {
                    before,
                    counts,
                    tickets: self.range_tickets.len(),
                },
            });
        }
        self.custody_registry().reconcile_releases();
        Ok(())
    }

    pub(in crate::store) fn replace_range_record(
        &mut self,
        id: DomRangeId,
        record: RangeRecord<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.range_record(id)?;
        let mut slot = self.ranges.slot(id.slot).ok_or(DomError::StaleRange)?;
        slot.record = Some(record);
        self.write_range(id.slot, slot, cancel)
    }

    pub(in crate::store) fn update_active_range_dependency(
        &mut self,
        document: Key,
        old: DomActiveRangeState,
        new: DomActiveRangeState,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let document = self.canonical_key(document);
        if old == new
            || (!matches!(old, DomActiveRangeState::Known(Some(_)))
                && !matches!(new, DomActiveRangeState::Known(Some(_))))
        {
            return Ok(());
        }
        let retain = self.prepare_undo(cancel)?;
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut candidate = self.ranges.current.clone();
        let registry = self.custody_registry();
        let mut admit = |usage| self.admit_custody_usage(usage, cancel);
        let mut step = || self.source_step(cancel);
        if let DomActiveRangeState::Known(Some(id)) = old {
            candidate.association(document, id, false, registry, &mut admit, &mut step)?;
        }
        if let DomActiveRangeState::Known(Some(id)) = new {
            self.range_record(id)?;
            candidate.association(document, id, true, registry, &mut admit, &mut step)?;
        }
        match document {
            Key::Existing(id) => set(
                &mut candidate.active_documents,
                &mut candidate.active_documents_custody,
                id.slot,
                id,
                false,
                registry,
                &mut admit,
            )?,
            Key::Created(index) => set(
                &mut candidate.active_created_documents,
                &mut candidate.active_created_documents_custody,
                index,
                index,
                false,
                registry,
                &mut admit,
            )?,
        }
        let before = std::mem::replace(&mut self.ranges.current, candidate);
        if retain {
            self.record_range_undo_prepared(RangeUndo {
                change: RangeChange::State {
                    before,
                    counts: self.counts,
                    tickets: self.range_tickets.len(),
                },
            });
        }
        self.custody_registry().reconcile_releases();
        Ok(())
    }

    pub(in crate::store) fn range_participants(
        &self,
        container: Key,
        cancel: &AtomicBool,
    ) -> Result<Participants, DomError> {
        let mut participants = Participants::new(self.custody_registry());
        self.add_range_participants(container, &mut participants, cancel)?;
        Ok(participants)
    }

    pub(in crate::store) fn add_range_participants(
        &self,
        container: Key,
        participants: &mut Participants,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let container = self.canonical_key(container);
        self.source_step(cancel)?;
        let bank = match container {
            Key::Existing(_) => &self.ranges.current.dependencies.containers,
            Key::Created(_) => &self.ranges.current.dependencies.created_containers,
        };
        let index = match container {
            Key::Existing(id) => id.slot,
            Key::Created(index) => index,
        };
        if let Some(bucket) = bank.get(index).filter(|bucket| bucket.key == container) {
            members(
                &self.ranges.current.dependencies.nodes,
                bucket.root,
                &mut || self.source_step(cancel),
                &mut |member| {
                    let Member::Range(id) = member else {
                        unreachable!("endpoint membership contains Ranges")
                    };
                    participants.insert(id, self, cancel)
                },
            )?;
        }
        Ok(())
    }

    pub(in crate::store) fn range_associated_documents(
        &self,
        id: DomRangeId,
        cancel: &AtomicBool,
    ) -> Result<custody::Held<Vec<Key>>, DomError> {
        let mut documents = custody::Held::new(
            Vec::new(),
            self.custody_ledger()
                .retain(custody::Usage::ZERO, |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?,
        );
        if let Some(association) = self
            .ranges
            .current
            .dependencies
            .associations
            .get(id.slot)
            .filter(|entry| entry.id == id)
        {
            for root in [association.existing, association.created]
                .into_iter()
                .flatten()
            {
                members(
                    &self.ranges.current.dependencies.nodes,
                    root,
                    &mut || self.source_step(cancel),
                    &mut |member| {
                        let Member::Document(document) = member else {
                            unreachable!("active association contains Documents")
                        };
                        self.range_temporary_push(&mut documents, self.live_key(document), cancel)
                    },
                )?;
            }
        }
        Ok(documents)
    }

    pub(in crate::store) fn range_temporary_push<T: Copy>(
        &self,
        values: &mut custody::Held<Vec<T>>,
        value: T,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if values.value().len() == values.value().capacity() {
            let capacity = values
                .value()
                .len()
                .checked_add(1)
                .and_then(|minimum| minimum.checked_next_power_of_two())
                .ok_or(DomError::RepresentationExhausted)?;
            let bytes = capacity
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?;
            let debit = self
                .custody_ledger()
                .retain(custody::Usage::new(bytes, 0), |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?;
            let mut next = custody::Held::new(Vec::new(), debit);
            next.value_mut()
                .try_reserve_exact(capacity)
                .map_err(|_| DomError::AllocationFailed)?;
            let actual = next
                .value()
                .capacity()
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?;
            #[cfg(test)]
            allocated(actual);
            if actual > bytes {
                next.debit_mut()
                    .grow(custody::Usage::new(actual - bytes, 0), |usage| {
                        self.admit_custody_usage(usage, cancel)
                    })?;
            }
            for value in values.value() {
                self.source_step(cancel)?;
                next.value_mut().push(*value);
                #[cfg(test)]
                super::sequence::record_copied_bytes(size_of::<T>());
            }
            *values = next;
        }
        values.value_mut().push(value);
        Ok(())
    }

    pub(in crate::store) fn reserve_range_ticket(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.range_tickets.len() < self.range_tickets.capacity() {
            return Ok(());
        }
        let retain = self.prepare_undo(cancel)?;
        let capacity = self
            .range_tickets
            .len()
            .checked_add(1)
            .and_then(|minimum| minimum.checked_next_power_of_two())
            .ok_or(DomError::RepresentationExhausted)?;
        let bytes = capacity
            .checked_mul(size_of::<DomRangeId>())
            .ok_or(DomError::RepresentationExhausted)?;
        let debit = self
            .custody_ledger()
            .retain(custody::Usage::new(bytes, 0), |usage| {
                self.admit_custody_usage(usage, cancel)
            })?;
        let mut next = custody::Held::new(Vec::new(), debit);
        next.value_mut()
            .try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        let actual = next
            .value()
            .capacity()
            .checked_mul(size_of::<DomRangeId>())
            .ok_or(DomError::RepresentationExhausted)?;
        #[cfg(test)]
        allocated(actual);
        if actual > bytes {
            next.debit_mut()
                .grow(custody::Usage::new(actual - bytes, 0), |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?;
        }
        for id in &self.range_tickets {
            self.source_step(cancel)?;
            next.value_mut().push(*id);
            #[cfg(test)]
            super::sequence::record_copied_bytes(size_of::<DomRangeId>());
        }
        let (values, debit) = next.into_parts();
        let before = custody::Held::new(
            std::mem::replace(&mut self.range_tickets, values),
            std::mem::replace(&mut self.range_tickets_charge, debit),
        );
        if retain {
            self.record_range_undo_prepared(RangeUndo {
                change: RangeChange::Tickets { before },
            });
        }
        Ok(())
    }

    pub(in crate::store) fn finish_ranges(
        &self,
        mapper: &impl Fn(Key) -> NodeId,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        let _cleanup = Reconcile(self.custody_registry().clone());
        let mut output = Pair {
            semantic: self.ranges.base.clone(),
            custody: self.ranges.base_custody.clone(),
        };
        let mut candidate = self.ranges.current.clone();
        let registry = self.custody_registry();
        let mut admit = |usage| self.admit_custody_usage(usage, cancel);
        let mut step = || self.source_step(cancel);
        // Publication derives allocation metadata from the original canonical
        // basis plus surviving changes, discarding unpublished serials/tails.
        candidate.dependencies.availability = self.ranges.base_dependencies.availability.clone();
        candidate.dependency_custody.availability =
            self.ranges.base_dependency_custody.availability.clone();
        let mut extent = self.ranges.base.len();
        let mut range_changed = false;
        for (index, slot) in self.ranges.current.overlay.iter() {
            self.source_step(cancel)?;
            if slot.record.is_some() {
                extent = extent.max(
                    index
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                );
            }
        }
        for (index, slot) in self.ranges.current.overlay.iter() {
            self.source_step(cancel)?;
            if index >= extent {
                continue;
            }
            let old = self.ranges.base.get(index);
            let (generation, retired) = if slot.record.is_none() {
                old.map_or((0, false), |old| {
                    (
                        old.generation,
                        old.retired || (old.record.is_some() && old.generation == u64::MAX),
                    )
                })
            } else {
                (slot.generation, slot.retired)
            };
            let next = RangeSlot {
                generation,
                retired,
                record: slot.record.map(|record| record.map(mapper)),
            };
            if old != Some(&next) {
                range_changed = true;
                set(
                    &mut output.semantic,
                    &mut output.custody,
                    index,
                    next.clone(),
                    reusable(&next),
                    registry,
                    &mut admit,
                )?;
            }
            if old.is_none_or(|old| reusable(old) != reusable(&next)) {
                set(
                    &mut candidate.dependencies.availability,
                    &mut candidate.dependency_custody.availability,
                    index,
                    (),
                    reusable(&next),
                    registry,
                    &mut admit,
                )?;
            }
        }
        let active = |state| match state {
            DomActiveRangeState::Known(id) => id,
            DomActiveRangeState::Unavailable => None,
        };
        let mut active_changed = false;
        for document in self.ranges.current.active_documents.values().copied() {
            self.source_step(cancel)?;
            let before = self
                .base
                .node(self.store, document)
                .ok()
                .and_then(|node| active(node.active_range));
            let after = self
                .node(Key::Existing(document))
                .ok()
                .and_then(|node| active(node.active_range));
            active_changed |= before != after;
        }
        for index in self
            .ranges
            .current
            .active_created_documents
            .values()
            .copied()
        {
            self.source_step(cancel)?;
            active_changed |= self
                .node(Key::Created(index))
                .ok()
                .and_then(|node| active(node.active_range))
                .is_some();
        }
        let dependencies_changed = range_changed || active_changed;
        if !dependencies_changed {
            // Intermediate Range allocation/retirement can return to the same
            // canonical facts. Retain the original derived index backings.
            candidate.dependencies = self.ranges.base_dependencies.clone();
            candidate.dependency_custody = self.ranges.base_dependency_custody.clone();
        }
        // Only newly introduced container keys require identity translation.
        for (index, bucket) in self
            .ranges
            .current
            .dependencies
            .created_containers
            .iter()
            .filter(|_| dependencies_changed)
        {
            self.source_step(cancel)?;
            let id = mapper(bucket.key);
            set(
                &mut candidate.dependencies.containers,
                &mut candidate.dependency_custody.containers,
                id.slot,
                Container {
                    key: Key::Existing(id),
                    root: bucket.root,
                },
                false,
                registry,
                &mut admit,
            )?;
            remove(
                &mut candidate.dependencies.created_containers,
                &mut candidate.dependency_custody.created_containers,
                index,
                registry,
                &mut admit,
            )?;
        }
        // Existing Document memberships are already canonical. Reconcile only
        // the created memberships in association roots changed in this Draft.
        for id in self
            .ranges
            .current
            .active_touched
            .values()
            .copied()
            .filter(|_| dependencies_changed)
        {
            let Some(association) = self
                .ranges
                .current
                .dependencies
                .associations
                .get(id.slot)
                .filter(|entry| entry.id == id)
            else {
                continue;
            };
            let Some(root) = association.created else {
                continue;
            };
            let mut documents = custody::Held::new(
                Vec::new(),
                registry.ledger().retain(custody::Usage::ZERO, &mut admit)?,
            );
            members(
                &self.ranges.current.dependencies.nodes,
                root,
                &mut step,
                &mut |member| {
                    let Member::Document(document) = member else {
                        unreachable!("association root contains Documents")
                    };
                    self.range_temporary_push(&mut documents, document, cancel)
                },
            )?;
            for document in documents.value() {
                candidate.association(*document, id, false, registry, &mut admit, &mut step)?;
                candidate.association(
                    Key::Existing(mapper(*document)),
                    id,
                    true,
                    registry,
                    &mut admit,
                    &mut step,
                )?;
            }
        }
        // Move the semantic outputs into the finished holder before custody.
        let ranges = std::mem::take(&mut output.semantic);
        let custody = std::mem::take(&mut output.custody);
        Ok(Finished {
            ranges,
            dependencies: candidate.dependencies,
            custody,
            dependency_custody: candidate.dependency_custody,
        })
    }
}
