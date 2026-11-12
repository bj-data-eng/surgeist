use super::*;
use crate::DomChildListRecord;

/// References captured at the exact source algorithm position, before any
/// later tree edits can change a sibling or retire an added or removed node.
#[derive(Clone, Copy)]
pub(in crate::store) struct MutationInput<'a> {
    pub target: Key,
    pub added: &'a [Key],
    pub removed: &'a [Key],
    pub previous: Option<Key>,
    pub next: Option<Key>,
}

impl<'a> MutationInput<'a> {
    fn keys(self) -> impl Iterator<Item = Key> + 'a {
        std::iter::once(self.target)
            .chain(self.previous)
            .chain(self.next)
            .chain(self.added.iter().copied())
            .chain(self.removed.iter().copied())
    }

    fn form(
        self,
        project: impl Fn(Key) -> Result<Key, DomError>,
        edit: &Edit<'_>,
    ) -> Result<(Arc<Mutation>, bank::ObjectCustody), DomError> {
        let identity = custody::MaterializationId::fresh()?;
        let mut debit = edit
            .draft
            .custody_ledger()
            .retain(custody::Usage::new(size_of::<Mutation>(), 0), |usage| {
                edit.admit(usage)
            })?;
        let added = copy(self.added.iter().copied().map(&project), &mut debit, edit)?;
        let removed = copy(self.removed.iter().copied().map(&project), &mut debit, edit)?;
        let mutation = Mutation {
            identity,
            target: project(self.target)?,
            previous: self.previous.map(&project).transpose()?,
            next: self.next.map(&project).transpose()?,
            added,
            removed,
        };
        let debit = debit.share(|usage| edit.admit(usage))?;
        let custody =
            edit.draft
                .custody_registry()
                .adopt_object_retained(identity, &debit, |usage| edit.admit(usage))?;
        Ok((Arc::new(mutation), custody))
    }
}

fn copy<T>(
    values: impl ExactSizeIterator<Item = Result<T, DomError>>,
    debit: &mut custody::Debit,
    edit: &Edit<'_>,
) -> Result<SharedVec<T>, DomError> {
    let count = values.len();
    if count == 0 {
        return Ok(SharedVec::default());
    }
    let bytes = count
        .checked_mul(size_of::<T>())
        .and_then(|bytes| bytes.checked_add(size_of::<Vec<T>>()))
        .ok_or(DomError::RepresentationExhausted)?;
    debit.grow(custody::Usage::new(bytes, 0), |usage| edit.admit(usage))?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(count)
        .map_err(|_| DomError::AllocationFailed)?;
    let extra = (result.capacity() - count)
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    debit.grow(custody::Usage::new(extra, 0), |usage| edit.admit(usage))?;
    for value in values {
        edit.step()?;
        result.push(value?);
    }
    Ok(result.into())
}

impl Mutation {
    fn input(&self) -> MutationInput<'_> {
        MutationInput {
            target: self.target,
            added: &self.added,
            removed: &self.removed,
            previous: self.previous,
            next: self.next,
        }
    }

    fn output(
        &self,
        debit: &mut custody::Debit,
        edit: &Edit<'_>,
    ) -> Result<DomChildListRecord, DomError> {
        let public = |key| match key {
            Key::Existing(node) => Ok(node),
            Key::Created(_) => Err(DomError::ChildListPublicationUnavailable),
        };
        Ok(DomChildListRecord {
            target: public(self.target)?,
            added: copy(self.added.iter().copied().map(public), debit, edit)?,
            removed: copy(self.removed.iter().copied().map(public), debit, edit)?,
            previous: self.previous.map(public).transpose()?,
            next: self.next.map(public).transpose()?,
        })
    }
}

impl Working {
    fn write_body(&mut self, index: usize, body: Body, edit: &Edit<'_>) -> Result<(), DomError> {
        let description = body.describe(edit.draft.custody_registry(), &mut |usage| {
            edit.admit(usage)
        })?;
        edit.write_described(
            &mut self.tables.bodies,
            &mut self.controls.bodies,
            index,
            body,
            false,
            description,
        )
    }

    pub(super) fn queue_record(
        &mut self,
        input: MutationInput<'_>,
        edit: &Edit<'_>,
    ) -> Result<bool, DomError> {
        if input.added.is_empty() && input.removed.is_empty() {
            return Ok(false);
        }
        let mut reached = bank::Bank::default();
        let mut reach_custody = bank::BankCustody::default();
        let body_index = self
            .tables
            .bodies
            .lowest_vacant()
            .unwrap_or(self.tables.bodies.len());
        let mut reached_body = None;
        let mut occurrences = 0usize;
        let mut ancestor = Some(input.target);
        while let Some(node) = ancestor {
            edit.step()?;
            let mut registration = self
                .tables
                .target(edit.target(node))
                .and_then(|target| target.registrations.first);
            while let Some(index) = registration {
                edit.step()?;
                let row = self.registration(index)?;
                registration = row.by_target.next;
                if node != input.target && !row.subtree || reached.get(row.observer.slot).is_some()
                {
                    continue;
                }
                edit.write(
                    &mut reached,
                    &mut reach_custody,
                    row.observer.slot,
                    true,
                    false,
                )?;
                if reached_body.is_none() {
                    reached_body = Some(input.form(|key| Ok(edit.target(key)), edit)?);
                }
                let (mutation, _) = reached_body
                    .as_ref()
                    .ok_or(DomError::ChildListPublicationUnavailable)?;
                self.append_record(row.observer, body_index, mutation.references()?, edit)?;
                self.mark_pending(row.observer, edit)?;
                occurrences = occurrences
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
            ancestor = edit.draft.node(node)?.parent();
        }
        let Some((mutation, _custody)) = reached_body else {
            return Ok(false);
        };
        let mut provisional = false;
        for key in mutation.input().keys() {
            edit.step()?;
            if let Key::Created(index) = key {
                provisional = true;
                if self.observed_creations.get(index).is_none() {
                    edit.write(
                        &mut self.observed_creations,
                        &mut self.observed_creation_controls,
                        index,
                        true,
                        false,
                    )?;
                }
            }
        }
        if provisional {
            edit.write(
                &mut self.provisional_bodies,
                &mut self.provisional_body_controls,
                body_index,
                true,
                false,
            )?;
        }
        self.write_body(
            body_index,
            Body {
                mutation,
                occurrences,
            },
            edit,
        )?;
        Ok(true)
    }

    fn append_record(
        &mut self,
        id: DomChildListObserverId,
        body: usize,
        references: usize,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let mut observer = self.tables.observer(edit.draft.store, id)?;
        let records = self
            .tables
            .counts
            .records
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        let references = self
            .tables
            .counts
            .references
            .checked_add(references)
            .ok_or(DomError::RepresentationExhausted)?;
        let limits = edit.draft.dom_limits.child_list_observation;
        admit(records, limits.records, DomResource::ChildListRecords)?;
        admit(
            references,
            limits.references,
            DomResource::ChildListRecordReferences,
        )?;
        let index = self
            .tables
            .records
            .lowest_vacant()
            .unwrap_or(self.tables.records.len());
        if let Some(last) = observer.records.last {
            let mut entry = self
                .tables
                .records
                .get(last)
                .copied()
                .ok_or(DomError::ChildListPublicationUnavailable)?;
            entry.next = Some(index);
            edit.write(
                &mut self.tables.records,
                &mut self.controls.records,
                last,
                entry,
                false,
            )?;
        } else {
            observer.records.first = Some(index);
        }
        observer.records.last = Some(index);
        edit.write(
            &mut self.tables.records,
            &mut self.controls.records,
            index,
            QueuedRecord { body, next: None },
            false,
        )?;
        self.put_observer(id, Some(observer), edit)?;
        self.tables.counts.records = records;
        self.tables.counts.references = references;
        Ok(())
    }

    pub(super) fn drain_records(
        &mut self,
        id: DomChildListObserverId,
        mut output: Option<&mut custody::Held<Vec<DomChildListRecord>>>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let mut observer = self.tables.observer(edit.draft.store, id)?;
        let mut record = observer.records.first;
        if record.is_none() {
            return Ok(());
        }
        while let Some(index) = record {
            edit.step()?;
            let entry = self
                .tables
                .records
                .get(index)
                .copied()
                .ok_or(DomError::ChildListPublicationUnavailable)?;
            let body = self
                .tables
                .bodies
                .get(entry.body)
                .ok_or(DomError::ChildListPublicationUnavailable)?
                .clone();
            let references = body.mutation.references()?;
            if let Some(output) = output.as_deref_mut() {
                push_output(output, &body.mutation, edit)?;
            }
            if body.occurrences == 1 {
                edit.remove(
                    &mut self.tables.bodies,
                    &mut self.controls.bodies,
                    entry.body,
                )?;
            } else {
                self.write_body(
                    entry.body,
                    Body {
                        occurrences: body
                            .occurrences
                            .checked_sub(1)
                            .ok_or(DomError::RepresentationExhausted)?,
                        ..body
                    },
                    edit,
                )?;
            }
            edit.remove(&mut self.tables.records, &mut self.controls.records, index)?;
            self.tables.counts.records = self
                .tables
                .counts
                .records
                .checked_sub(1)
                .ok_or(DomError::RepresentationExhausted)?;
            self.tables.counts.references = self
                .tables
                .counts
                .references
                .checked_sub(references)
                .ok_or(DomError::RepresentationExhausted)?;
            record = entry.next;
        }
        observer.records = List::default();
        self.put_observer(id, Some(observer), edit)
    }

    pub(super) fn publish_bodies(
        &mut self,
        identify: &impl Fn(Key) -> Option<NodeId>,
        edit: &Edit<'_>,
    ) -> Result<(), DomError> {
        let provisional = self.provisional_bodies.clone();
        for (index, _) in provisional.iter() {
            edit.step()?;
            let Some(body) = self.tables.bodies.get(index) else {
                continue;
            };
            let (mutation, _custody) = body.mutation.input().form(
                |key| {
                    identify(key)
                        .map(Key::Existing)
                        .ok_or(DomError::ChildListPublicationUnavailable)
                },
                edit,
            )?;
            let occurrences = body.occurrences;
            self.write_body(
                index,
                Body {
                    mutation,
                    occurrences,
                },
                edit,
            )?;
        }
        Ok(())
    }
}

fn push_output(
    output: &mut custody::Held<Vec<DomChildListRecord>>,
    mutation: &Mutation,
    edit: &Edit<'_>,
) -> Result<(), DomError> {
    reserve_output(output, edit)?;
    let record = mutation.output(output.debit_mut(), edit)?;
    output.value_mut().push(record);
    Ok(())
}

pub(super) fn reserve_output<T>(
    output: &mut custody::Held<Vec<T>>,
    edit: &Edit<'_>,
) -> Result<(), DomError> {
    if output.value().len() == output.value().capacity() {
        let before = output.value().capacity();
        let next = before
            .checked_mul(2)
            .ok_or(DomError::RepresentationExhausted)?
            .max(1);
        let bytes = next
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut backing = edit
            .draft
            .custody_ledger()
            .retain(custody::Usage::new(bytes, 0), |usage| edit.admit(usage))?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(next)
            .map_err(|_| DomError::AllocationFailed)?;
        let extra = (values.capacity() - next)
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        backing.grow(custody::Usage::new(extra, 0), |usage| edit.admit(usage))?;
        for _ in 0..output.value().len() {
            edit.step()?;
        }
        values.append(output.value_mut());
        let old_bytes = before
            .checked_mul(size_of::<T>())
            .ok_or(DomError::RepresentationExhausted)?;
        let old = std::mem::replace(output.value_mut(), values);
        drop(old);
        drop(
            output
                .debit_mut()
                .split(custody::Usage::new(old_bytes, 0))?,
        );
        output
            .debit_mut()
            .absorb(backing)
            .map_err(|_| DomError::RepresentationExhausted)?;
    }
    Ok(())
}
