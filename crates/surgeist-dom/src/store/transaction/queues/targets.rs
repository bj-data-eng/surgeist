//! Exact queue membership summaries. Canonical snapshots keep semantic roots;
//! the exclusive owner retains their paired custody independently.
use super::associations::{AssociationsCustody, PublishedTarget, QueueAssociations};
use super::*;

#[derive(Debug)]
pub(in crate::store) struct Target<K> {
    target: K,
    count: usize,
}
#[cfg(test)]
type NodeCounts = Vec<(NodeId, usize)>;
#[derive(Clone, Debug, Default)]
pub(in crate::store) struct CanonicalTargets {
    targets: bank::Bank<Target<NodeId>>,
    documents: bank::Bank<Target<NodeId>>,
    associations: QueueAssociations,
}
#[derive(Clone, Default)]
pub(in crate::store) struct TargetsCustody {
    targets: bank::BankCustody<Target<NodeId>>,
    documents: bank::BankCustody<Target<NodeId>>,
    associations: AssociationsCustody,
}
impl CanonicalTargets {
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<TargetsCustody, DomError> {
        let mut describe = |_: &Target<NodeId>| {
            step()?;
            Ok(custody::Usage::new(size_of::<Target<NodeId>>(), 0))
        };
        let targets = self
            .targets
            .adopt_controlled(registry, &mut describe, &mut *admit)?;
        let documents = self
            .documents
            .adopt_controlled(registry, &mut describe, &mut *admit)?;
        let associations = self.associations.adopt_custody(registry, admit, step)?;
        Ok(TargetsCustody {
            targets,
            documents,
            associations,
        })
    }
    #[cfg(test)]
    pub fn totals(&self) -> bank::Totals {
        let a = self.targets.totals();
        let b = self.documents.totals();
        let c = self.associations.totals();
        bank::Totals {
            bytes: a
                .bytes
                .checked_add(b.bytes)
                .and_then(|value| value.checked_add(c.bytes))
                .expect("admitted queue target payload union"),
            units: a
                .units
                .checked_add(b.units)
                .and_then(|value| value.checked_add(c.units))
                .expect("admitted queue target units"),
            occurrences: a
                .occurrences
                .checked_add(b.occurrences)
                .and_then(|value| value.checked_add(c.occurrences))
                .expect("admitted queue target records"),
        }
    }
    #[cfg(test)]
    pub fn index_charge(&self) -> bank::IndexCharge {
        let a = self.targets.index_charge();
        let b = self.documents.index_charge();
        let c = self.associations.index_charge();
        bank::IndexCharge {
            bytes: a
                .bytes
                .checked_add(b.bytes)
                .and_then(|value| value.checked_add(c.bytes))
                .expect("admitted queue target index union"),
            pages: a
                .pages
                .checked_add(b.pages)
                .and_then(|value| value.checked_add(c.pages))
                .expect("admitted queue target pages"),
        }
    }
    pub fn has_document_obligations(&self, document: DocumentId) -> bool {
        self.documents
            .get(document.node().slot)
            .is_some_and(|record| record.target == document.node() && record.count != 0)
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        let describe = |_: &Target<NodeId>| {
            Ok(bank::Totals {
                bytes: size_of::<Target<NodeId>>(),
                units: 0,
                occurrences: 1,
            })
        };
        let mut graph = self.targets.inspect_graph(describe)?;
        graph.merge(&self.documents.inspect_graph(describe)?)?;
        graph.merge(&self.associations.inspect_graph()?)?;
        Ok(graph)
    }
    #[cfg(test)]
    pub fn inspect_counts(&self) -> (NodeCounts, NodeCounts) {
        (
            self.targets
                .iter()
                .map(|(_, value)| (value.target, value.count))
                .collect(),
            self.documents
                .iter()
                .map(|(_, value)| (value.target, value.count))
                .collect(),
        )
    }
    #[cfg(test)]
    pub fn shares_root(&self, other: &Self) -> bool {
        self.targets.shares_root(&other.targets)
            && self.documents.shares_root(&other.documents)
            && self.associations.shares_root(&other.associations)
    }
    #[cfg(test)]
    pub fn inspect_associations(&self) -> Vec<(Key, Key, usize)> {
        self.associations.inspect_associations()
    }
}

// Tombstones retain the complete local identity until publication. Existing
// slots and Created aliases remain separate domains, including slot reuse.
#[derive(Clone, Default)]
struct Dirty {
    existing: bank::Bank<Target<NodeId>>,
    created: bank::Bank<Target<usize>>,
    existing_custody: bank::BankCustody<Target<NodeId>>,
    created_custody: bank::BankCustody<Target<usize>>,
}
impl Dirty {
    fn mark(
        &mut self,
        key: Key,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        match key {
            Key::Existing(id) => {
                if self
                    .existing
                    .get(id.slot)
                    .is_some_and(|entry| entry.target == id)
                {
                    return Ok(());
                }
                change(
                    &mut self.existing,
                    &mut self.existing_custody,
                    id,
                    1,
                    true,
                    registry,
                    admit,
                )
            }
            Key::Created(index) => {
                if self.created.get(index).is_some() {
                    return Ok(());
                }
                change(
                    &mut self.created,
                    &mut self.created_custody,
                    index,
                    1,
                    true,
                    registry,
                    admit,
                )
            }
        }
    }
    fn keys(&self) -> impl Iterator<Item = Key> + '_ {
        self.existing
            .iter()
            .map(|(_, entry)| Key::Existing(entry.target))
            .chain(
                self.created
                    .iter()
                    .map(|(_, entry)| Key::Created(entry.target)),
            )
    }
}

pub(super) struct Targets {
    existing: bank::Bank<Target<NodeId>>,
    documents: bank::Bank<Target<NodeId>>,
    created_documents: bank::Bank<Target<usize>>,
    created: bank::Bank<Target<usize>>,
    associations: QueueAssociations,
    existing_custody: bank::BankCustody<Target<NodeId>>,
    documents_custody: bank::BankCustody<Target<NodeId>>,
    created_documents_custody: bank::BankCustody<Target<usize>>,
    created_custody: bank::BankCustody<Target<usize>>,
    associations_custody: AssociationsCustody,
    dirty_targets: Dirty,
    dirty_documents: Dirty,
    published: Option<FinishedTargets>,
    registry: bank::CustodyRegistry,
}
impl Clone for Targets {
    fn clone(&self) -> Self {
        Self {
            existing: self.existing.clone(),
            documents: self.documents.clone(),
            created_documents: self.created_documents.clone(),
            created: self.created.clone(),
            associations: self.associations.clone(),
            existing_custody: self.existing_custody.clone(),
            documents_custody: self.documents_custody.clone(),
            created_documents_custody: self.created_documents_custody.clone(),
            created_custody: self.created_custody.clone(),
            associations_custody: self.associations_custody.clone(),
            dirty_targets: self.dirty_targets.clone(),
            dirty_documents: self.dirty_documents.clone(),
            published: self.published.clone(),
            registry: self.registry.clone(),
        }
    }
}
impl Drop for Targets {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.existing));
        drop(std::mem::take(&mut self.created));
        drop(std::mem::take(&mut self.documents));
        drop(std::mem::take(&mut self.created_documents));
        drop(std::mem::take(&mut self.associations));
        drop(std::mem::take(&mut self.existing_custody));
        drop(std::mem::take(&mut self.created_custody));
        drop(std::mem::take(&mut self.documents_custody));
        drop(std::mem::take(&mut self.created_documents_custody));
        drop(std::mem::take(&mut self.associations_custody));
        drop(std::mem::take(&mut self.dirty_targets));
        drop(std::mem::take(&mut self.dirty_documents));
        drop(self.published.take());
        self.registry.reconcile_releases();
    }
}
pub(in crate::store) struct FinishedTargets {
    records: CanonicalTargets,
    custody: TargetsCustody,
    registry: bank::CustodyRegistry,
}
impl Clone for FinishedTargets {
    fn clone(&self) -> Self {
        Self {
            records: self.records.clone(),
            custody: self.custody.clone(),
            registry: self.registry.clone(),
        }
    }
}
impl FinishedTargets {
    pub fn into_parts(mut self) -> (CanonicalTargets, TargetsCustody) {
        (
            std::mem::take(&mut self.records),
            std::mem::take(&mut self.custody),
        )
    }
}
impl Drop for FinishedTargets {
    fn drop(&mut self) {
        drop(std::mem::take(&mut self.records));
        drop(std::mem::take(&mut self.custody));
        self.registry.reconcile_releases();
    }
}
// Derive storage addressing from the complete target identity rather than
// accepting a separately supplied slot that could disagree with that identity.
trait TargetIndex: Copy + Eq {
    fn target_index(self) -> usize;
}
impl TargetIndex for NodeId {
    fn target_index(self) -> usize {
        self.slot
    }
}
impl TargetIndex for usize {
    fn target_index(self) -> usize {
        self
    }
}
fn change<K: TargetIndex>(
    records: &mut bank::Bank<Target<K>>,
    custody: &mut bank::BankCustody<Target<K>>,
    target: K,
    amount: usize,
    add: bool,
    registry: &bank::CustodyRegistry,
    mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let index = target.target_index();
    let previous = records.get(index);
    if previous.is_some_and(|record| record.target != target) {
        return Err(DomError::StaleNode);
    }
    let count = previous.map_or(0, |record| record.count);
    let count = if add {
        count.checked_add(amount)
    } else {
        count.checked_sub(amount)
    }
    .ok_or(DomError::RepresentationExhausted)?;
    let (next, next_custody) = if count == 0 {
        let (next, custody, removed) =
            records.remove_controlled(custody, index, registry, &mut admit)?;
        drop(removed);
        (next, custody)
    } else {
        let usage = custody::Usage::new(size_of::<Target<K>>(), 0);
        let record =
            bank::ControlledRecord::new(Target { target, count }, usage, registry, &mut admit)?;
        records.set_controlled(
            custody,
            index,
            record,
            false,
            bank::Totals {
                bytes: usage.bytes(),
                units: 0,
                occurrences: 1,
            },
            registry,
            &mut admit,
        )?
    };
    *records = next;
    *custody = next_custody;
    registry.reconcile_releases();
    Ok(())
}
fn absolute(
    records: &mut bank::Bank<Target<NodeId>>,
    custody: &mut bank::BankCustody<Target<NodeId>>,
    target: NodeId,
    count: usize,
    registry: &bank::CustodyRegistry,
    admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let before = records.get(target.slot);
    if count == 0 && before.is_some_and(|entry| entry.target != target) {
        return Ok(());
    }
    if before.is_some_and(|entry| entry.target != target) {
        return Err(DomError::StaleNode);
    }
    let old = before.map_or(0, |entry| entry.count);
    if old == count {
        return Ok(());
    }
    change(
        records,
        custody,
        target,
        old.abs_diff(count),
        count > old,
        registry,
        admit,
    )
}
impl Targets {
    pub fn capture(
        records: &CanonicalTargets,
        custody: &TargetsCustody,
        registry: &bank::CustodyRegistry,
    ) -> Self {
        Self {
            existing: records.targets.clone(),
            documents: records.documents.clone(),
            created_documents: bank::Bank::default(),
            created: bank::Bank::default(),
            associations: records.associations.clone(),
            existing_custody: custody.targets.clone(),
            documents_custody: custody.documents.clone(),
            created_documents_custody: bank::BankCustody::default(),
            created_custody: bank::BankCustody::default(),
            associations_custody: custody.associations.clone(),
            dirty_targets: Dirty::default(),
            dirty_documents: Dirty::default(),
            published: Some(FinishedTargets {
                records: records.clone(),
                custody: custody.clone(),
                registry: registry.clone(),
            }),
            registry: registry.clone(),
        }
    }
    pub fn count(&self, target: Key) -> usize {
        match target {
            Key::Existing(id) => self
                .existing
                .get(id.slot)
                .filter(|record| record.target == id)
                .map_or(0, |record| record.count),
            Key::Created(index) => self
                .created
                .get(index)
                .filter(|record| record.target == index)
                .map_or(0, |record| record.count),
        }
    }
    pub fn updated(
        &self,
        keys: impl IntoIterator<Item = Result<(Key, Option<Key>), DomError>>,
        add: bool,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut candidate = self.clone();
        for entry in keys {
            let (key, document) = entry?;
            candidate
                .dirty_targets
                .mark(key, &candidate.registry, &mut admit)?;
            if let Some(document) = document {
                candidate.change_document(document, 1, add, &mut admit)?;
            }
            step()?;
            match key {
                Key::Existing(id) => change(
                    &mut candidate.existing,
                    &mut candidate.existing_custody,
                    id,
                    1,
                    add,
                    &candidate.registry,
                    &mut admit,
                )?,
                Key::Created(index) => change(
                    &mut candidate.created,
                    &mut candidate.created_custody,
                    index,
                    1,
                    add,
                    &candidate.registry,
                    &mut admit,
                )?,
            }
        }
        Ok(candidate)
    }
    pub fn finish(
        &self,
        id: &impl Fn(Key) -> NodeId,
        private: impl Fn(Key) -> bool,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<FinishedTargets, DomError> {
        let mut finished = self
            .published
            .as_ref()
            .expect("canonical queue metadata base")
            .clone();
        for key in self.dirty_targets.keys() {
            step()?;
            if private(key) {
                continue;
            }
            let target = match key {
                Key::Existing(id) => id,
                Key::Created(_) => id(key),
            };
            absolute(
                &mut finished.records.targets,
                &mut finished.custody.targets,
                target,
                self.count(key),
                &finished.registry,
                &mut admit,
            )?;
            let associations = finished.records.associations.replace_published_target(
                &finished.custody.associations,
                PublishedTarget {
                    id: target,
                    associations: &self.associations,
                    key,
                },
                id,
                &finished.registry,
                &mut admit,
                &mut step,
            )?;
            let previous =
                std::mem::replace(&mut finished.records.associations, associations.value);
            let previous_custody =
                std::mem::replace(&mut finished.custody.associations, associations.custody);
            drop(previous);
            drop(previous_custody);
            finished.registry.reconcile_releases();
        }
        for key in self.dirty_documents.keys() {
            step()?;
            if private(key) {
                continue;
            }
            let document = match key {
                Key::Existing(id) => id,
                Key::Created(_) => id(key),
            };
            absolute(
                &mut finished.records.documents,
                &mut finished.custody.documents,
                document,
                self.document_count(key),
                &finished.registry,
                &mut admit,
            )?;
        }
        Ok(finished)
    }
    fn document_count(&self, document: Key) -> usize {
        match document {
            Key::Existing(id) => self
                .documents
                .get(id.slot)
                .filter(|record| record.target == id)
                .map_or(0, |record| record.count),
            Key::Created(index) => self
                .created_documents
                .get(index)
                .map_or(0, |record| record.count),
        }
    }
    pub fn touch_target(
        &mut self,
        target: Key,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.dirty_targets.mark(target, &self.registry, admit)
    }
    fn change_document(
        &mut self,
        document: Key,
        count: usize,
        add: bool,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.dirty_documents.mark(document, &self.registry, admit)?;
        match document {
            Key::Existing(id) => change(
                &mut self.documents,
                &mut self.documents_custody,
                id,
                count,
                add,
                &self.registry,
                admit,
            ),
            Key::Created(index) => change(
                &mut self.created_documents,
                &mut self.created_documents_custody,
                index,
                count,
                add,
                &self.registry,
                admit,
            ),
        }
    }
    pub fn move_document(
        &self,
        target: Key,
        old: Key,
        new: Key,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<Self>, DomError> {
        let count = self.count(target);
        if count == 0 || old == new {
            return Ok(None);
        }
        let mut candidate = self.clone();
        step()?;
        candidate.change_document(old, count, false, &mut admit)?;
        step()?;
        candidate.change_document(new, count, true, &mut admit)?;
        Ok(Some(candidate))
    }
    pub fn change_documents(
        &self,
        documents: impl IntoIterator<Item = Result<(Key, usize), DomError>>,
        add: bool,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut candidate = self.clone();
        for document in documents {
            let (document, count) = document?;
            step()?;
            candidate.change_document(document, count, add, &mut admit)?;
        }
        Ok(candidate)
    }
    pub fn change_associations(
        &self,
        associations: impl IntoIterator<Item = Result<(Key, Key, usize), DomError>>,
        add: bool,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut candidate = self.clone();
        for association in associations {
            let (target, document, count) = association?;
            candidate
                .dirty_targets
                .mark(target, &candidate.registry, &mut admit)?;
            let changed = candidate.associations.changed(
                &candidate.associations_custody,
                transaction::counted_relations::Change {
                    group: target,
                    member: document,
                    delta: if add {
                        transaction::counted_relations::Delta::Add(count)
                    } else {
                        transaction::counted_relations::Delta::Remove(count)
                    },
                },
                &candidate.registry,
                &mut admit,
                &mut step,
            )?;
            let previous = std::mem::replace(&mut candidate.associations, changed.value);
            let previous_custody =
                std::mem::replace(&mut candidate.associations_custody, changed.custody);
            drop(previous);
            drop(previous_custody);
            candidate.registry.reconcile_releases();
        }
        Ok(candidate)
    }
    pub fn change_associated_documents(
        &self,
        target: Key,
        map: &impl Fn(Key) -> Key,
        add: bool,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut candidate = self.clone();
        let admission = std::cell::RefCell::new(admit);
        let work = std::cell::RefCell::new(step);
        self.associations.visit_documents(
            target,
            &mut |document, count| {
                candidate.change_document(map(document), count, add, &mut **admission.borrow_mut())
            },
            &mut || (**work.borrow_mut())(),
        )?;
        Ok(candidate)
    }
    pub fn mark_published(&mut self, records: &CanonicalTargets, custody: &TargetsCustody) {
        drop(std::mem::take(&mut self.dirty_targets));
        drop(std::mem::take(&mut self.dirty_documents));
        self.published = Some(FinishedTargets {
            records: records.clone(),
            custody: custody.clone(),
            registry: self.registry.clone(),
        });
    }
}
