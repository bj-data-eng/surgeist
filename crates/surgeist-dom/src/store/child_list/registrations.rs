use super::*;

impl Working {
    pub(super) fn create_observer(
        &mut self,
        edit: &Edit<'_>,
    ) -> Result<DomChildListObserverId, DomError> {
        let index = self
            .tables
            .observers
            .lowest_vacant()
            .unwrap_or(self.tables.observers.len());
        let generation = match self.tables.observers.get(index) {
            Some(slot) => slot
                .generation
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            None => 1,
        };
        let count = self.tables.observers.len().max(
            index
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
        );
        admit(
            count,
            edit.draft.dom_limits.child_list_observation.observers,
            DomResource::ChildListObservers,
        )?;
        let id = DomChildListObserverId {
            owner: edit.draft.store,
            slot: index,
            generation,
        };
        self.put_observer(id, Some(Observer::default()), edit)?;
        Ok(id)
    }

    pub(super) fn put_observer(
        &mut self,
        id: DomChildListObserverId,
        observer: Option<Observer>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        edit.write(
            &mut self.tables.observers,
            &mut self.controls.observers,
            id.slot,
            ObserverSlot {
                generation: id.generation,
                value: observer,
            },
            observer.is_none() && id.generation < u64::MAX,
        )
    }

    fn put_target(
        &mut self,
        node: Key,
        registrations: List,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let (index, values, custody) = match node {
            Key::Existing(id) => (
                id.slot,
                &mut self.tables.targets,
                &mut self.controls.targets,
            ),
            Key::Created(index) => (
                index,
                &mut self.tables.created_targets,
                &mut self.controls.created_targets,
            ),
        };
        if registrations.first.is_some() {
            edit.write(
                values,
                custody,
                index,
                Target {
                    node,
                    registrations,
                },
                false,
            )
        } else {
            edit.remove(values, custody, index)
        }
    }

    fn put_registration(
        &mut self,
        index: usize,
        value: Registration,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        edit.write(
            &mut self.tables.registrations,
            &mut self.controls.registrations,
            index,
            value,
            false,
        )
    }

    pub(super) fn registration(&self, index: usize) -> Result<Registration, DomError> {
        self.tables
            .registrations
            .get(index)
            .copied()
            .ok_or(DomError::StaleChildListObserver)
    }

    fn append_registration(
        &mut self,
        id: DomChildListObserverId,
        target: Key,
        subtree: bool,
        source: Option<usize>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let count = self
            .tables
            .counts
            .registrations
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            count,
            edit.draft.dom_limits.child_list_observation.registrations,
            DomResource::ChildListRegistrations,
        )?;
        let index = self
            .tables
            .registrations
            .lowest_vacant()
            .unwrap_or(self.tables.registrations.len());
        let mut observer = self.tables.observer(edit.draft.store, id)?;
        let mut target_list = self
            .tables
            .target(target)
            .map_or_else(List::default, |value| value.registrations);
        let registration = Registration {
            observer: id,
            target,
            subtree,
            source,
            by_target: Links {
                previous: target_list.last,
                next: None,
            },
            by_observer: Links {
                previous: None,
                next: observer.registrations,
            },
        };
        if let Some(last) = target_list.last {
            let mut row = self.registration(last)?;
            row.by_target.next = Some(index);
            self.put_registration(last, row, edit)?;
        } else {
            target_list.first = Some(index);
        }
        if let Some(first) = observer.registrations {
            let mut row = self.registration(first)?;
            row.by_observer.previous = Some(index);
            self.put_registration(first, row, edit)?;
        }
        target_list.last = Some(index);
        observer.registrations = Some(index);
        self.put_registration(index, registration, edit)?;
        self.put_target(target, target_list, edit)?;
        self.put_observer(id, Some(observer), edit)?;
        self.tables.counts.registrations = count;
        Ok(())
    }

    fn unlink_registration(&mut self, index: usize, edit: &Edit<'_>) -> Result<(), DomError> {
        let row = self.registration(index)?;
        let mut observer = self.tables.observer(edit.draft.store, row.observer)?;
        let mut target = self
            .tables
            .target(row.target)
            .ok_or(DomError::StaleChildListObserver)?
            .registrations;
        if let Some(previous) = row.by_target.previous {
            let mut entry = self.registration(previous)?;
            entry.by_target.next = row.by_target.next;
            self.put_registration(previous, entry, edit)?;
        } else {
            target.first = row.by_target.next;
        }
        if let Some(next) = row.by_target.next {
            let mut entry = self.registration(next)?;
            entry.by_target.previous = row.by_target.previous;
            self.put_registration(next, entry, edit)?;
        } else {
            target.last = row.by_target.previous;
        }
        if let Some(previous) = row.by_observer.previous {
            let mut entry = self.registration(previous)?;
            entry.by_observer.next = row.by_observer.next;
            self.put_registration(previous, entry, edit)?;
        } else {
            observer.registrations = row.by_observer.next;
        }
        if let Some(next) = row.by_observer.next {
            let mut entry = self.registration(next)?;
            entry.by_observer.previous = row.by_observer.previous;
            self.put_registration(next, entry, edit)?;
        }
        self.put_target(row.target, target, edit)?;
        self.put_observer(row.observer, Some(observer), edit)?;
        edit.remove(
            &mut self.tables.registrations,
            &mut self.controls.registrations,
            index,
        )?;
        self.tables.counts.registrations = self
            .tables
            .counts
            .registrations
            .checked_sub(1)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(())
    }

    pub(super) fn clear_transients(
        &mut self,
        id: DomChildListObserverId,
        source: Option<usize>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let mut registration = self.tables.observer(edit.draft.store, id)?.registrations;
        while let Some(index) = registration {
            edit.step()?;
            let row = self.registration(index)?;
            registration = row.by_observer.next;
            if row.source.is_some() && source.is_none_or(|source| row.source == Some(source)) {
                self.unlink_registration(index, edit)?;
            }
        }
        Ok(())
    }

    pub(super) fn observe(
        &mut self,
        id: DomChildListObserverId,
        target: Key,
        subtree: bool,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        self.tables.observer(edit.draft.store, id)?;
        let mut candidate = self
            .tables
            .target(target)
            .and_then(|target| target.registrations.first);
        while let Some(index) = candidate {
            edit.step()?;
            let row = self.registration(index)?;
            if row.observer == id && row.source.is_none() {
                self.clear_transients(id, Some(index), edit)?;
                let row = self.registration(index)?;
                if row.subtree != subtree {
                    self.put_registration(index, Registration { subtree, ..row }, edit)?;
                }
                return Ok(());
            }
            candidate = row.by_target.next;
        }
        self.append_registration(id, target, subtree, None, edit)
    }

    pub(super) fn disconnect_registrations(
        &mut self,
        id: DomChildListObserverId,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        while let Some(index) = self.tables.observer(edit.draft.store, id)?.registrations {
            self.unlink_registration(index, edit)?;
        }
        Ok(())
    }

    pub(super) fn retire_target(&mut self, target: Key, edit: &Edit<'_>) -> Result<(), DomError> {
        while let Some(index) = self
            .tables
            .target(target)
            .and_then(|target| target.registrations.first)
        {
            let row = self.registration(index)?;
            if row.source.is_none() {
                self.clear_transients(row.observer, Some(index), edit)?;
            }
            self.unlink_registration(index, edit)?;
        }
        Ok(())
    }

    pub(super) fn mark_pending(
        &mut self,
        id: DomChildListObserverId,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let mut observer = self.tables.observer(edit.draft.store, id)?;
        if observer.pending.is_some() {
            return Ok(());
        }
        let last = self.tables.pending.last;
        if let Some(index) = last {
            let slot = self
                .tables
                .observers
                .get(index)
                .copied()
                .ok_or(DomError::StaleChildListObserver)?;
            let mut earlier = slot.value.ok_or(DomError::StaleChildListObserver)?;
            earlier
                .pending
                .as_mut()
                .ok_or(DomError::StaleChildListObserver)?
                .next = Some(id.slot);
            let earlier_id = DomChildListObserverId {
                owner: edit.draft.store,
                slot: index,
                generation: slot.generation,
            };
            self.put_observer(earlier_id, Some(earlier), edit)?;
        } else {
            self.tables.pending.first = Some(id.slot);
        }
        self.tables.pending.last = Some(id.slot);
        observer.pending = Some(Links {
            previous: last,
            next: None,
        });
        self.put_observer(id, Some(observer), edit)
    }

    pub(super) fn add_transients(
        &mut self,
        node: Key,
        parent: Key,
        edit: &Edit<'_>,
    ) -> Result<bool, DomError> {
        let target = edit.target(node);
        let mut seen = bank::Bank::default();
        let mut custody = bank::BankCustody::default();
        let mut candidate = self
            .tables
            .target(target)
            .and_then(|target| target.registrations.first);
        while let Some(index) = candidate {
            edit.step()?;
            let row = self.registration(index)?;
            if let Some(source) = row.source {
                edit.write(&mut seen, &mut custody, source, true, false)?;
            }
            candidate = row.by_target.next;
        }
        let mut added = false;
        let mut ancestor = Some(parent);
        while let Some(node) = ancestor {
            edit.step()?;
            let mut candidate = self
                .tables
                .target(edit.target(node))
                .and_then(|target| target.registrations.first);
            while let Some(index) = candidate {
                edit.step()?;
                let row = self.registration(index)?;
                let source = row.source.unwrap_or(index);
                if row.subtree && seen.get(source).is_none() {
                    edit.write(&mut seen, &mut custody, source, true, false)?;
                    self.append_registration(row.observer, target, true, Some(source), edit)?;
                    self.mark_pending(row.observer, edit)?;
                    added = true;
                }
                candidate = row.by_target.next;
            }
            ancestor = edit.draft.node(node)?.parent();
        }
        Ok(added)
    }

    pub(super) fn publish_targets(
        &mut self,
        identify: &impl Fn(Key) -> Option<NodeId>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let created = self.tables.created_targets.clone();
        for (_, target) in created.iter() {
            edit.step()?;
            let node = identify(target.node)
                .map(Key::Existing)
                .ok_or(DomError::StaleNode)?;
            self.put_target(node, target.registrations, edit)?;
            let mut candidate = target.registrations.first;
            while let Some(index) = candidate {
                edit.step()?;
                let row = self.registration(index)?;
                self.put_registration(
                    index,
                    Registration {
                        target: node,
                        ..row
                    },
                    edit,
                )?;
                candidate = row.by_target.next;
            }
        }
        self.tables.created_targets = bank::Bank::default();
        self.controls.created_targets = bank::BankCustody::default();
        Ok(())
    }
}
