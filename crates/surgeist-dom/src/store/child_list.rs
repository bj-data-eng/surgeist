//! Canonical observer membership and ordered child-list record custody.

use super::*;
use crate::{DomChildListObservationLimits, DomChildListObserverId, DomChildListRecord};

mod checkpoint;
mod records;
mod registrations;
pub use checkpoint::DomChildListNotification;
pub(super) use checkpoint::{CheckpointWork, Turn};
pub(super) use records::MutationInput;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct List {
    first: Option<usize>,
    last: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Links {
    previous: Option<usize>,
    next: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Observer {
    registrations: Option<usize>,
    records: List,
    pending: Option<Links>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObserverSlot {
    generation: u64,
    value: Option<Observer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Registration {
    observer: DomChildListObserverId,
    target: Key,
    subtree: bool,
    /// The direct registration from which this transient was derived.
    source: Option<usize>,
    by_target: Links,
    by_observer: Links,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Target {
    node: Key,
    registrations: List,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct QueuedRecord {
    body: usize,
    next: Option<usize>,
}

#[derive(Debug, Eq, PartialEq)]
struct Mutation {
    identity: custody::MaterializationId,
    target: Key,
    added: SharedVec<Key>,
    removed: SharedVec<Key>,
    previous: Option<Key>,
    next: Option<Key>,
}

impl Mutation {
    fn references(&self) -> Result<usize, DomError> {
        self.added
            .len()
            .checked_add(self.removed.len())
            .and_then(|count| {
                count.checked_add(
                    1 + usize::from(self.previous.is_some()) + usize::from(self.next.is_some()),
                )
            })
            .ok_or(DomError::RepresentationExhausted)
    }

    fn usage(&self) -> Result<custody::Usage, DomError> {
        let bytes = self
            .added
            .retained_bytes()?
            .checked_add(self.removed.retained_bytes()?)
            .and_then(|bytes| bytes.checked_add(size_of::<Self>()))
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(custody::Usage::new(bytes, 0))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Body {
    mutation: Arc<Mutation>,
    occurrences: usize,
}

impl Body {
    fn describe(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<bank::RecordDescription, DomError> {
        let custody =
            registry.adopt_object(self.mutation.identity, self.mutation.usage()?, &mut *admit)?;
        let dependency = bank::Dependency::new(custody, registry.ledger(), admit)?;
        Ok(bank::RecordDescription::with_dependency(
            custody::Usage::new(size_of::<Self>(), 0),
            dependency,
        ))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct Counts {
    registrations: usize,
    records: usize,
    references: usize,
}

/// The membership and record topology contains only semantic values. Matching
/// custody is retained by the owner separately from immutable snapshots.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct Tables {
    observers: bank::Bank<ObserverSlot>,
    targets: bank::Bank<Target>,
    created_targets: bank::Bank<Target>,
    registrations: bank::Bank<Registration>,
    records: bank::Bank<QueuedRecord>,
    bodies: bank::Bank<Body>,
    pending: List,
    counts: Counts,
}

#[derive(Clone, Default)]
pub(super) struct Controls {
    observers: bank::BankCustody<ObserverSlot>,
    targets: bank::BankCustody<Target>,
    created_targets: bank::BankCustody<Target>,
    registrations: bank::BankCustody<Registration>,
    records: bank::BankCustody<QueuedRecord>,
    bodies: bank::BankCustody<Body>,
}

#[derive(Debug)]
pub(super) struct Published {
    identity: custody::MaterializationId,
    tables: Tables,
}

impl PartialEq for Published {
    fn eq(&self, other: &Self) -> bool {
        self.tables == other.tables
    }
}

pub(super) type PublishedCustody = bank::Attached<Controls>;

impl Published {
    fn usage() -> custody::Usage {
        custody::Usage::new(size_of::<Self>(), 0)
    }

    pub(super) fn admit(&self, limits: DomChildListObservationLimits) -> Result<(), DomError> {
        self.tables.admit(limits)
    }

    pub(super) fn adopt(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<PublishedCustody, DomError> {
        let root = registry.adopt_object(self.identity, Self::usage(), &mut *admit)?;
        if let Some(existing) = root.attachment() {
            return Ok(existing);
        }
        let tables = &self.tables;
        let bodies = {
            let admission = std::cell::RefCell::new(&mut *admit);
            tables.bodies.adopt_controlled(
                registry,
                |body| {
                    step()?;
                    body.describe(registry, &mut |usage| (*admission.borrow_mut())(usage))
                },
                |usage| (*admission.borrow_mut())(usage),
            )?
        };
        let controls = Controls {
            bodies,
            observers: adopt_table(&tables.observers, registry, admit, step)?,
            targets: adopt_table(&tables.targets, registry, admit, step)?,
            created_targets: adopt_table(&tables.created_targets, registry, admit, step)?,
            registrations: adopt_table(&tables.registrations, registry, admit, step)?,
            records: adopt_table(&tables.records, registry, admit, step)?,
        };
        attach_controls(root, controls, registry.ledger(), admit)
    }
}

fn adopt_table<T>(
    values: &bank::Bank<T>,
    registry: &bank::CustodyRegistry,
    admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
    step: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<bank::BankCustody<T>, DomError> {
    values.adopt_controlled(
        registry,
        |_| {
            step()?;
            Ok(custody::Usage::new(size_of::<T>(), 0))
        },
        admit,
    )
}

fn attach_controls(
    root: bank::ObjectCustody,
    controls: Controls,
    ledger: &custody::Ledger,
    admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<PublishedCustody, DomError> {
    let debit = ledger.retain(custody::Usage::new(size_of::<Controls>(), 0), admit)?;
    Ok(root.attach(custody::Held::new(Arc::new(controls), debit)))
}

#[derive(Clone, Default)]
struct Working {
    tables: Tables,
    controls: Controls,
    provisional_bodies: bank::Bank<bool>,
    provisional_body_controls: bank::BankCustody<bool>,
    observed_creations: bank::Bank<bool>,
    observed_creation_controls: bank::BankCustody<bool>,
}

pub(super) struct Version {
    working: Working,
    _storage: custody::Debit,
}

pub(super) struct Undo(Option<Arc<Version>>);

impl Undo {
    pub(super) fn restore(self, draft: &mut Draft) {
        draft.child_lists = self.0;
    }
}

impl Draft {
    fn child_lists(&self) -> Option<&Tables> {
        self.child_lists
            .as_deref()
            .map(|current| &current.working.tables)
            .or_else(|| {
                self.base
                    .child_lists
                    .as_deref()
                    .map(|published| &published.tables)
            })
    }

    fn edit_child_lists<T>(
        &mut self,
        cancel: &AtomicBool,
        change: impl FnOnce(&mut Working, &Edit<'_>) -> Result<T, DomError>,
    ) -> Result<T, DomError> {
        self.edit_child_lists_with_work(cancel, None, change)
    }

    fn edit_child_lists_with_work<T>(
        &mut self,
        cancel: &AtomicBool,
        checkpoint: Option<&CheckpointWork>,
        change: impl FnOnce(&mut Working, &Edit<'_>) -> Result<T, DomError>,
    ) -> Result<T, DomError> {
        let edit = Edit {
            draft: self,
            cancel,
            checkpoint,
        };
        let mut working = self.child_lists.as_deref().map_or_else(
            || Working {
                tables: self
                    .base
                    .child_lists
                    .as_deref()
                    .map(|value| value.tables.clone())
                    .unwrap_or_default(),
                controls: self
                    .base_custody
                    .child_lists
                    .as_deref()
                    .cloned()
                    .unwrap_or_default(),
                ..Working::default()
            },
            |current| current.working.clone(),
        );
        let output = change(&mut working, &edit)?;
        working
            .tables
            .admit(self.dom_limits.child_list_observation)?;
        let storage = self
            .custody_ledger()
            .retain(custody::Usage::new(size_of::<Version>(), 0), |usage| {
                edit.admit(usage)
            })?;
        let retain_undo = self.prepare_undo(cancel)?;
        let previous = self.child_lists.replace(Arc::new(Version {
            working,
            _storage: storage,
        }));
        if retain_undo {
            self.journal
                .push_prepared(transaction::Undo::ChildList(Undo(previous)));
        }
        Ok(output)
    }

    pub(super) fn child_list_exposes_creation(&self, index: usize) -> bool {
        self.child_lists
            .as_ref()
            .is_some_and(|version| version.working.observed_creations.get(index).is_some())
    }

    pub(super) fn has_child_list_registrations(&self) -> bool {
        self.child_lists()
            .is_some_and(|tables| tables.counts.registrations != 0)
    }

    pub(super) fn may_observe_child_list(&self) -> bool {
        self.has_child_list_registrations()
            || (self.source_limits.is_some()
                && self.dom_limits.child_list_observation.registrations != 0)
    }

    pub(super) fn child_list_position(
        &self,
        parent: Key,
        child: Key,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        self.source_step(cancel)?;
        let child = self.live_key(child);
        let parent = self.live_key(parent);
        self.slot_index
            .position(child)
            .filter(|position| position.parent == parent)
            .map(|position| position.ordinal)
            .ok_or(DomError::NotFound)
    }

    pub(super) fn child_list_before(
        &self,
        parent: Key,
        child: Option<Key>,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        let children = self.node(parent)?.children();
        let index = match child {
            Some(child) => self.child_list_position(parent, child, cancel)?,
            None => children.len(),
        };
        Ok(index
            .checked_sub(1)
            .and_then(|previous| children.get(previous)))
    }

    pub(super) fn child_list_removed(
        &mut self,
        node: Key,
        parent: Key,
        siblings: Option<(Option<Key>, Option<Key>)>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.has_child_list_registrations() {
            return Ok(());
        }
        let pending = self.edit_child_lists(cancel, |working, edit| {
            let transients = working.add_transients(node, parent, edit)?;
            let mut queued = false;
            if let Some((previous, next)) = siblings {
                queued = working.queue_record(
                    MutationInput {
                        target: parent,
                        added: &[],
                        removed: &[node],
                        previous,
                        next,
                    },
                    edit,
                )?;
            }
            Ok(transients || queued)
        })?;
        if pending {
            self.queue_mutation_notification(cancel)?;
        }
        Ok(())
    }

    pub(super) fn queue_child_list_record(
        &mut self,
        input: MutationInput<'_>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.has_child_list_registrations()
            && self.edit_child_lists(cancel, |working, edit| working.queue_record(input, edit))?
        {
            self.queue_mutation_notification(cancel)?;
        }
        Ok(())
    }

    pub(super) fn retire_child_list_target(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let key = self.published_id(node).map_or(node, Key::Existing);
        if self
            .child_lists()
            .and_then(|tables| tables.target(key))
            .is_none()
        {
            return Ok(());
        }
        self.edit_child_lists(cancel, |working, edit| working.retire_target(key, edit))
    }

    pub(super) fn finish_child_lists(
        &self,
        identify: impl Fn(Key) -> Option<NodeId>,
        cancel: &AtomicBool,
    ) -> Result<Finished, DomError> {
        let unchanged = || Finished {
            value: self.base.child_lists.clone(),
            custody: self.base_custody.child_lists.clone(),
            changed: false,
        };
        let Some(current) = &self.child_lists else {
            return Ok(unchanged());
        };
        let edit = Edit {
            draft: self,
            cancel,
            checkpoint: None,
        };
        let mut working = current.working.clone();
        working.publish_targets(&identify, &edit)?;
        working.publish_bodies(&identify, &edit)?;
        if self
            .base
            .child_lists
            .as_ref()
            .is_some_and(|base| base.tables == working.tables)
        {
            return Ok(unchanged());
        }
        let identity = custody::MaterializationId::fresh()?;
        let registry = self.custody_registry();
        let root =
            registry.adopt_object(identity, Published::usage(), |usage| edit.admit(usage))?;
        let value = Arc::new(Published {
            identity,
            tables: working.tables,
        });
        let custody = attach_controls(root, working.controls, registry.ledger(), |usage| {
            edit.admit(usage)
        })?;
        Ok(Finished {
            value: Some(value),
            custody: Some(custody),
            changed: true,
        })
    }
}

pub(super) struct Finished {
    pub(super) value: Option<Arc<Published>>,
    pub(super) custody: Option<PublishedCustody>,
    pub(super) changed: bool,
}

impl Dom {
    pub fn create_child_list_observer(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomChildListObserverId, DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.edit_child_lists(cancel, |working, edit| working.create_observer(edit))
        })
    }

    /// Observe the target's child list, optionally including its descendants.
    pub fn observe_child_list(
        &mut self,
        observer: DomChildListObserverId,
        target: NodeId,
        subtree: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| {
            let target = draft.resolve(target.into())?;
            draft.edit_child_lists(cancel, |working, edit| {
                working.observe(observer, edit.target(target), subtree, edit)
            })
        })
    }

    /// Empty the current record queue. Pending notification and transient
    /// registrations retain their independent lifetime.
    pub fn take_child_list_records(
        &mut self,
        observer: DomChildListObserverId,
        cancel: &AtomicBool,
    ) -> Result<Vec<DomChildListRecord>, DomError> {
        let output = self.edit_native_state(cancel, |draft| {
            let mut output = custody::Held::new(
                Vec::new(),
                draft
                    .custody_ledger()
                    .retain(custody::Usage::ZERO, |usage| {
                        draft.admit_custody_usage(usage, cancel)
                    })?,
            );
            draft.edit_child_lists(cancel, |working, edit| {
                working.drain_records(observer, Some(&mut output), edit)
            })?;
            Ok(output)
        })?;
        let (records, debit) = output.into_parts();
        drop(debit);
        Ok(records)
    }

    /// Remove all registrations and discard current records. The observer can
    /// observe another target with the same identity.
    pub fn disconnect_child_list_observer(
        &mut self,
        observer: DomChildListObserverId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.edit_child_lists(cancel, |working, edit| {
                working.disconnect_registrations(observer, edit)?;
                working.drain_records(observer, None, edit)
            })
        })
    }

    /// Release the native observer, its registrations, and its queued records.
    pub fn retire_child_list_observer(
        &mut self,
        observer: DomChildListObserverId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.edit_native_state(cancel, |draft| {
            draft.edit_child_lists(cancel, |working, edit| {
                working.disconnect_registrations(observer, edit)?;
                working.drain_records(observer, None, edit)?;
                working.unlink_pending(observer, edit)?;
                working.put_observer(observer, None, edit)
            })
        })
    }
}

impl Tables {
    fn admit(&self, limits: DomChildListObservationLimits) -> Result<(), DomError> {
        for (count, limit, resource) in [
            (
                self.observers.len(),
                limits.observers,
                DomResource::ChildListObservers,
            ),
            (
                self.counts.registrations,
                limits.registrations,
                DomResource::ChildListRegistrations,
            ),
            (
                self.counts.records,
                limits.records,
                DomResource::ChildListRecords,
            ),
            (
                self.counts.references,
                limits.references,
                DomResource::ChildListRecordReferences,
            ),
        ] {
            admit(count, limit, resource)?;
        }
        Ok(())
    }

    fn observer(
        &self,
        owner: DomOwnerId,
        id: DomChildListObserverId,
    ) -> Result<Observer, DomError> {
        if id.owner != owner {
            return Err(DomError::ForeignOwner);
        }
        self.observers
            .get(id.slot)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.value)
            .ok_or(DomError::StaleChildListObserver)
    }

    fn target(&self, node: Key) -> Option<Target> {
        match node {
            Key::Existing(id) => self.targets.get(id.slot),
            Key::Created(index) => self.created_targets.get(index),
        }
        .copied()
        .filter(|target| target.node == node)
    }
}

struct Edit<'a> {
    draft: &'a Draft,
    cancel: &'a AtomicBool,
    checkpoint: Option<&'a CheckpointWork>,
}

impl Edit<'_> {
    fn step(&self) -> Result<(), DomError> {
        self.draft.source_step(self.cancel)?;
        if let Some(work) = self.checkpoint {
            work.step(self.cancel)?;
        }
        Ok(())
    }

    fn admit(&self, usage: custody::Usage) -> Result<(), DomError> {
        self.draft.admit_custody_usage(usage, self.cancel)
    }

    fn target(&self, key: Key) -> Key {
        self.draft.published_id(key).map_or(key, Key::Existing)
    }

    fn write<T>(
        &self,
        values: &mut bank::Bank<T>,
        custody: &mut bank::BankCustody<T>,
        index: usize,
        value: T,
        reusable: bool,
    ) -> Result<(), DomError> {
        let usage = custody::Usage::new(size_of::<T>(), 0);
        self.write_described(values, custody, index, value, reusable, usage.into())
    }

    fn write_described<T>(
        &self,
        values: &mut bank::Bank<T>,
        custody: &mut bank::BankCustody<T>,
        index: usize,
        value: T,
        reusable: bool,
        description: bank::RecordDescription,
    ) -> Result<(), DomError> {
        self.step()?;
        let registry = self.draft.custody_registry();
        let record =
            bank::ControlledRecord::new(value, description, registry, |usage| self.admit(usage))?;
        let totals = bank::Totals {
            bytes: size_of::<T>(),
            units: 0,
            occurrences: 1,
        };
        (*values, *custody) = values.set_controlled(
            custody,
            index,
            record,
            reusable,
            totals,
            registry,
            |usage| self.admit(usage),
        )?;
        Ok(())
    }

    fn remove<T>(
        &self,
        values: &mut bank::Bank<T>,
        custody: &mut bank::BankCustody<T>,
        index: usize,
    ) -> Result<(), DomError> {
        self.step()?;
        let (next, controls, retired) =
            values.remove_controlled(custody, index, self.draft.custody_registry(), |usage| {
                self.admit(usage)
            })?;
        *values = next;
        *custody = controls;
        drop(retired);
        Ok(())
    }
}
