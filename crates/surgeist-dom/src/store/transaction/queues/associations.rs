//! Original task-document associations, grouped by the exact queued target.
//! Private rows retain their associations; visibility only changes public
//! document summaries. Canonical roots contain fully published node identities.
use super::*;
use transaction::counted_relations;

#[derive(Clone, Debug, Default)]
pub(super) struct QueueAssociations(counted_relations::Index);
#[derive(Clone, Default)]
pub(super) struct AssociationsCustody(counted_relations::Custody);
pub(super) struct Changed {
    pub value: QueueAssociations,
    pub custody: AssociationsCustody,
}
/// Replace one local target's association group at its published identity.
pub(super) struct PublishedTarget<'a> {
    pub id: NodeId,
    pub associations: &'a QueueAssociations,
    pub key: Key,
}
struct Reconcile<'a>(&'a bank::CustodyRegistry);
impl Drop for Reconcile<'_> {
    fn drop(&mut self) {
        self.0.reconcile_releases();
    }
}
impl Changed {
    fn apply(
        &mut self,
        change: counted_relations::Change,
        registry: &bank::CustodyRegistry,
        admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let changed = self
            .value
            .0
            .change(&self.custody.0, change, registry, admit, step)?;
        let previous = std::mem::replace(&mut self.value.0, changed.value);
        let previous_custody = std::mem::replace(&mut self.custody.0, changed.custody);
        drop(previous);
        drop(previous_custody);
        registry.reconcile_releases();
        Ok(())
    }
}
impl QueueAssociations {
    pub fn adopt_custody(
        &self,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<AssociationsCustody, DomError> {
        self.0
            .adopt_custody(registry, admit, step)
            .map(AssociationsCustody)
    }
    pub fn changed(
        &self,
        custody: &AssociationsCustody,
        change: counted_relations::Change,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Changed, DomError> {
        let _reconcile = Reconcile(registry);
        let mut candidate = Changed {
            value: self.clone(),
            custody: custody.clone(),
        };
        candidate.apply(change, registry, &mut admit, &mut step)?;
        Ok(candidate)
    }
    pub fn visit_documents(
        &self,
        target: Key,
        visit: &mut impl FnMut(Key, usize) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.0.visit_group(target, visit, step)
    }
    pub fn replace_published_target(
        &self,
        custody: &AssociationsCustody,
        target: PublishedTarget<'_>,
        id: &impl Fn(Key) -> NodeId,
        registry: &bank::CustodyRegistry,
        admit: &mut impl FnMut(custody::Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<Changed, DomError> {
        let _reconcile = Reconcile(registry);
        let mut candidate = Changed {
            value: self.clone(),
            custody: custody.clone(),
        };
        // This alias keeps old semantic records and their exact custody alive
        // while the selected canonical group is replaced on persistent paths.
        let previous = Changed {
            value: self.clone(),
            custody: custody.clone(),
        };
        let admission = std::cell::RefCell::new(admit);
        let work = std::cell::RefCell::new(step);
        previous.value.visit_documents(
            Key::Existing(target.id),
            &mut |document, count| {
                candidate.apply(
                    counted_relations::Change {
                        group: Key::Existing(target.id),
                        member: document,
                        delta: counted_relations::Delta::Remove(count),
                    },
                    registry,
                    &mut **admission.borrow_mut(),
                    &mut **work.borrow_mut(),
                )
            },
            &mut || (**work.borrow_mut())(),
        )?;
        target.associations.visit_documents(
            target.key,
            &mut |document, count| {
                let document = match document {
                    Key::Existing(id) => id,
                    Key::Created(_) => id(document),
                };
                candidate.apply(
                    counted_relations::Change {
                        group: Key::Existing(target.id),
                        member: Key::Existing(document),
                        delta: counted_relations::Delta::Add(count),
                    },
                    registry,
                    &mut **admission.borrow_mut(),
                    &mut **work.borrow_mut(),
                )
            },
            &mut || (**work.borrow_mut())(),
        )?;
        Ok(candidate)
    }
    #[cfg(test)]
    pub fn totals(&self) -> bank::Totals {
        self.0.totals()
    }
    #[cfg(test)]
    pub fn index_charge(&self) -> bank::IndexCharge {
        self.0.index_charge()
    }
    #[cfg(test)]
    pub fn inspect_graph(&self) -> Result<bank::PhysicalGraph, DomError> {
        self.0.inspect_graph()
    }
    #[cfg(test)]
    pub fn inspect_associations(&self) -> Vec<(Key, Key, usize)> {
        self.0.inspect_relations()
    }
    #[cfg(test)]
    pub fn shares_root(&self, other: &Self) -> bool {
        self.0.shares_root(&other.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn accept(_: custody::Usage) -> Result<(), DomError> {
        Ok(())
    }
    #[test]
    fn selected_target_preserves_original_documents_multiplicity_and_publication_aliases() {
        let registry = bank::CustodyRegistry::new(&custody::Ledger::default(), accept).unwrap();
        let owner = crate::DomOwnerId::allocate().unwrap();
        let node = |slot, generation| NodeId {
            owner,
            slot,
            generation,
        };
        let target = Key::Created(7);
        let first = Key::Existing(node(1, 3));
        let second = Key::Created(2);
        let initial = QueueAssociations::default();
        let custody = AssociationsCustody::default();
        let first_change = initial
            .changed(
                &custody,
                counted_relations::Change {
                    group: target,
                    member: first,
                    delta: counted_relations::Delta::Add(2),
                },
                &registry,
                accept,
                || Ok(()),
            )
            .unwrap();
        let current = first_change
            .value
            .changed(
                &first_change.custody,
                counted_relations::Change {
                    group: target,
                    member: second,
                    delta: counted_relations::Delta::Add(1),
                },
                &registry,
                accept,
                || Ok(()),
            )
            .unwrap();
        let mut selected = Vec::new();
        current
            .value
            .visit_documents(
                target,
                &mut |document, count| {
                    selected.push((document, count));
                    Ok(())
                },
                &mut || Ok(()),
            )
            .unwrap();
        assert_eq!(selected.len(), 2);
        assert!(selected.contains(&(first, 2)));
        assert!(selected.contains(&(second, 1)));
        let mut visited = 0;
        current
            .value
            .visit_documents(
                Key::Existing(node(7, 4)),
                &mut |_, _| {
                    visited += 1;
                    Ok(())
                },
                &mut || Ok(()),
            )
            .unwrap();
        assert_eq!(
            visited, 0,
            "Created and Existing generations are distinct groups"
        );
        let mapped_target = node(20, 9);
        let mapped_doc = node(21, 8);
        let published = initial
            .replace_published_target(
                &custody,
                PublishedTarget {
                    id: mapped_target,
                    associations: &current.value,
                    key: target,
                },
                &|key| {
                    if key == second {
                        mapped_doc
                    } else {
                        panic!("unexpected remap")
                    }
                },
                &registry,
                &mut accept,
                &mut || Ok(()),
            )
            .unwrap();
        let rows = published.value.inspect_associations();
        assert_eq!(rows.len(), 2);
        assert!(rows.contains(&(Key::Existing(mapped_target), first, 2)));
        assert!(rows.contains(&(Key::Existing(mapped_target), Key::Existing(mapped_doc), 1)));
        let removed = current
            .value
            .changed(
                &current.custody,
                counted_relations::Change {
                    group: target,
                    member: first,
                    delta: counted_relations::Delta::Remove(2),
                },
                &registry,
                accept,
                || Ok(()),
            )
            .unwrap();
        let updated = published
            .value
            .replace_published_target(
                &published.custody,
                PublishedTarget {
                    id: mapped_target,
                    associations: &removed.value,
                    key: target,
                },
                &|key| {
                    if key == second {
                        mapped_doc
                    } else {
                        panic!("unexpected remap")
                    }
                },
                &registry,
                &mut accept,
                &mut || Ok(()),
            )
            .unwrap();
        assert_eq!(
            updated.value.inspect_associations(),
            vec![(Key::Existing(mapped_target), Key::Existing(mapped_doc), 1)]
        );
        assert_eq!(
            published.value.inspect_associations(),
            rows,
            "old canonical/savepoint remains immutable"
        );
        let before = registry.total();
        let refused = current.value.changed(
            &current.custody,
            counted_relations::Change {
                group: target,
                member: first,
                delta: counted_relations::Delta::Add(1),
            },
            &registry,
            |_| Err(DomError::Cancelled),
            || Ok(()),
        );
        assert!(matches!(refused, Err(DomError::Cancelled)));
        registry.reconcile_releases();
        assert_eq!(registry.total(), before);
        assert_eq!(current.value.inspect_associations().len(), 2);
    }
}
