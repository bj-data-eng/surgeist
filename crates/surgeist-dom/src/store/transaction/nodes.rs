//! Sparse record admission and selected field mutation.
use super::super::*;
use super::{
    Counts, ElementFieldsMut, NodeFields, NodeFieldsMut, NodeRead, RelationField, Sequence, Undo,
};

impl DraftNode {
    fn capture(
        node: &Node,
        ledger: &custody::Ledger,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let fields_debit = ledger.retain(node.receipts.fields, &mut admit)?;
        let fields = NodeFields::from_canonical(node);
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::RecordMaterialized {
                bytes: size_of::<NodeFields>(),
            });
        }
        let relations =
            |values: Option<&SharedVec<NodeId>>,
             admit: &mut dyn FnMut(custody::Usage) -> Result<(), DomError>| match values
            {
                Some(values) => Sequence::canonical(values, Key::Existing, ledger, admit),
                None => Sequence::empty(Key::Existing, ledger, admit),
            };
        let (candidates, open, top, pending, render) = (
            match &node.autofocus_document {
                Some(canonical::Autofocus::Known { candidates, .. }) => Some(candidates),
                _ => None,
            },
            match &node.interactive_document {
                Some(canonical::Interactive::Known { open_dialogs, .. }) => Some(open_dialogs),
                _ => None,
            },
            match &node.interactive_document {
                Some(canonical::Interactive::Known { top_layer, .. }) => Some(top_layer),
                _ => None,
            },
            match &node.interactive_document {
                Some(canonical::Interactive::Known {
                    pending_top_layer_removals,
                    ..
                }) => Some(pending_top_layer_removals),
                _ => None,
            },
            node.base_document.as_ref().map(|v| &v.render_blocking),
        );
        Ok(Self {
            fields: custody::Held::new(Box::new(fields), fields_debit),
            children: relations(Some(&node.children), &mut admit)?,
            manual_nodes: relations(Some(&node.manual_nodes), &mut admit)?,
            assigned_nodes: relations(Some(&node.assigned_nodes), &mut admit)?,
            fullscreen_events: Sequence::canonical(
                &node.fullscreen_events,
                std::convert::identity,
                ledger,
                &mut admit,
            )?,
            render_blocking: relations(render, &mut admit)?,
            autofocus_candidates: relations(candidates, &mut admit)?,
            open_dialogs: relations(open, &mut admit)?,
            top_layer: relations(top, &mut admit)?,
            pending_top_layer_removals: relations(pending, &mut admit)?,
        })
    }
    pub(super) fn relation_mut(&mut self, field: RelationField) -> &mut Sequence<NodeId, Key> {
        match field {
            RelationField::Children => &mut self.children,
            RelationField::ManualNodes => &mut self.manual_nodes,
            RelationField::AssignedNodes => &mut self.assigned_nodes,
            RelationField::RenderBlocking => &mut self.render_blocking,
            RelationField::AutofocusCandidates => &mut self.autofocus_candidates,
            RelationField::OpenDialogs => &mut self.open_dialogs,
            RelationField::TopLayer => &mut self.top_layer,
            RelationField::PendingTopLayerRemovals => &mut self.pending_top_layer_removals,
        }
    }
}

impl Draft {
    pub(in crate::store) fn private_node_mut(&mut self, key: Key) -> &mut DraftNode {
        let key = self.live_key(key);
        match key {
            Key::Existing(id) => self.records[*self
                .existing
                .get(id.slot)
                .expect("materialized record index")]
            .as_mut(),
            Key::Created(index) => self.created[index].as_mut(),
        }
        .expect("validated live private record")
    }
    fn reserve_record_arena(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        if self.records.len() < self.records.capacity() {
            return Ok(());
        }
        let undo = self.prepare_undo(cancel)?;
        let wanted = self
            .records
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?
            .max(self.records.capacity().saturating_mul(2));
        let ledger = self.custody_ledger().clone();
        let bytes = wanted
            .checked_mul(size_of::<Option<DraftNode>>())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut debit = ledger.retain(custody::Usage::new(bytes, 0), |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let mut next = Vec::new();
        next.try_reserve_exact(wanted)
            .map_err(|_| DomError::AllocationFailed)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: next.capacity() * size_of::<Option<DraftNode>>(),
            });
        }
        resize_debit(
            &mut debit,
            custody::Usage::new(
                next.capacity()
                    .checked_mul(size_of::<Option<DraftNode>>())
                    .ok_or(DomError::RepresentationExhausted)?,
                0,
            ),
            |usage| self.admit_custody_usage(usage, cancel),
        )?;
        for _ in &self.records {
            self.source_step(cancel)?;
        }
        #[cfg(test)]
        let moved_bytes = self.records.len() * size_of::<Option<DraftNode>>();
        next.append(&mut self.records);
        #[cfg(test)]
        super::sequence::record_copied_bytes(moved_bytes);
        let old = std::mem::replace(&mut self.records, next);
        let debit = std::mem::replace(&mut self.records_charge, debit);
        if undo {
            self.journal
                .push_prepared(Undo::ArenaCapacity { old, debit });
        } else {
            drop(old);
            drop(debit);
        }
        Ok(())
    }
    pub(in crate::store) fn materialize_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let key = self.live_key(key);
        self.node(key)?;
        let id = match key {
            Key::Existing(id) => id,
            Key::Created(index) => {
                if self.created[index].is_some() {
                    return Ok(());
                }
                let id = self
                    .phase_map
                    .as_ref()
                    .and_then(|m| m.ids.get(index))
                    .ok_or(DomError::StaleNode)?;
                let ledger = self.custody_ledger().clone();
                let node = DraftNode::capture(self.base.node(self.store, id)?, &ledger, |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?;
                self.set_created_dirty(index, true, cancel)?;
                let undo = self.prepare_undo(cancel)?;
                self.created[index] = Some(node);
                if undo {
                    self.journal
                        .push_prepared(Undo::CreatedMaterialized { index });
                }
                return Ok(());
            }
        };
        if self.existing.get(id.slot).is_some() {
            return Ok(());
        }
        self.reserve_record_arena(cancel)?;
        let undo = self.prepare_undo(cancel)?;
        let ledger = self.custody_ledger().clone();
        let node = DraftNode::capture(self.base.node(self.store, id)?, &ledger, |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let index = self.records.len();
        let record = bank::ControlledRecord::new(
            index,
            custody::Usage::new(size_of::<usize>(), 0),
            self.custody_registry(),
            |usage| self.admit_custody_usage(usage, cancel),
        )?;
        let (next, custody) = self.existing.set_controlled(
            &self.existing_custody,
            id.slot,
            record,
            false,
            bank::Totals {
                bytes: size_of::<usize>(),
                units: 0,
                occurrences: 1,
            },
            self.custody_registry(),
            |usage| self.admit_custody_usage(usage, cancel),
        )?;
        let before = std::mem::replace(&mut self.existing, next);
        let oldcustody = std::mem::replace(&mut self.existing_custody, custody);
        self.records.push(Some(node));
        #[cfg(test)]
        super::sequence::record_copied_bytes(size_of::<Option<DraftNode>>());
        if undo {
            self.journal.push_prepared(Undo::Materialized {
                before,
                custody: oldcustody,
                records: index,
            });
        }
        Ok(())
    }
    pub(in crate::store) fn node_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<NodeFieldsMut<'_>, DomError> {
        self.materialize_node(key, cancel)?;
        let undo = self.prepare_undo(cancel)?;
        #[cfg(test)]
        super::queues::admission_observation::before_fields(self, key);
        let usage = self.private_node_mut(key).fields.debit().usage();
        let debit = self
            .custody_ledger()
            .retain(usage, |usage| self.admit_custody_usage(usage, cancel))?;
        let mut value = Box::new((**self.private_node_mut(key).fields.value()).clone());
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::RecordMaterialized {
                bytes: size_of::<NodeFields>(),
            });
        }
        if let Some(map) = &self.phase_map {
            value.aliases(&map.ids);
        }
        Ok(NodeFieldsMut {
            draft: self,
            key,
            candidate: Some(custody::Held::new(value, debit)),
            retain_undo: undo,
        })
    }
    pub(in crate::store) fn element_fields_mut(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<ElementFieldsMut<'_>, DomError> {
        if !matches!(self.node(key)?.kind, DomNodeKind::Element(_)) {
            return Err(DomError::WrongNodeKind);
        }
        self.node_fields_mut(key, cancel).map(ElementFieldsMut)
    }
    pub(in crate::store) fn commit_node_fields(
        &mut self,
        key: Key,
        mut next: custody::Held<Box<NodeFields>>,
        undo: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let usage = next.value().usage(|| self.source_step(cancel))?;
        resize_debit(next.debit_mut(), usage, |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let undo = self.prepare_undo_slots(6, cancel)? && undo;
        let inbound = self.prepare_inbound_fields(key, next.value(), cancel)?;
        let controls = self.prepare_control_fields(key, next.value(), cancel)?;
        let slots = self.prepare_slot_fields(key, next.value(), cancel)?;
        let previous_document = self.node(key)?.document;
        let next_document = next.value().document;
        let queue_documents = if previous_document != next_document {
            Some(self.prepare_queue_document_change(
                key,
                previous_document,
                next_document,
                cancel,
            )?)
        } else {
            None
        };
        let previous_active = self.node(key)?.active_range;
        let next_active = next.value().active_range;
        let before = Counts::node(self.node(key)?)?;
        let after = Counts::node(NodeRead::staged_fields(
            self.private_node_mut(key),
            next.value(),
        ))?;
        let counts = self.counts.checked_replace(before, after)?;
        self.admit_node_counts(counts, self.nodes, self.units, self.dom_limits)?;
        let private = self.is_private(key);
        let next_private_units = if private {
            self.private_units
                .checked_sub(self.records_for(key).fields.value().logical_units()?)
                .and_then(|u| u.checked_add(next.value().logical_units().ok()?))
                .ok_or(DomError::RepresentationExhausted)?
        } else {
            self.private_units
        };
        let next_private_counts = if private {
            self.private_counts.checked_replace(before, after)?
        } else {
            self.private_counts
        };
        if previous_active != next_active {
            self.update_active_range_dependency(key, previous_active, next_active, cancel)?;
        }
        self.commit_inbound(inbound);
        if let Some(plan) = controls {
            self.commit_control_plan(plan);
        }
        let oldunits = self.units;
        let oldprivateunits = self.private_units;
        let oldprivatecounts = self.private_counts;
        self.private_units = next_private_units;
        self.private_counts = next_private_counts;
        let old = std::mem::replace(&mut self.private_node_mut(key).fields, next);
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        if let Some(plan) = queue_documents {
            self.commit_queue_document_change(plan);
        }
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        if undo {
            self.journal.push_prepared(Undo::Fields {
                key,
                old,
                counts: oldcounts,
                units: oldunits,
                private_units: oldprivateunits,
                private_counts: oldprivatecounts,
            });
        }
        Ok(())
    }
    pub(in crate::store) fn admit_node_counts(
        &self,
        mut counts: Counts,
        nodes: usize,
        units: usize,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        counts.select_tasks = self.native_tasks.counts().selected;
        counts.details_intentions = self.details_intentions.len();
        counts.slot_references = counts
            .slot_references
            .checked_add(self.signal_slots.len())
            .ok_or(DomError::RepresentationExhausted)?;
        counts.admit(self.ranges.len(), nodes, units, limits)
    }
    pub(in crate::store) fn admit_dom_resources(&self, limits: DomLimits) -> Result<(), DomError> {
        self.admit_node_counts(self.counts, self.nodes, self.units, limits)
    }
}
pub(in crate::store) fn resize_debit(
    debit: &mut custody::Debit,
    wanted: custody::Usage,
    mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let old = debit.usage();
    let extra = custody::Usage::new(
        wanted.bytes().saturating_sub(old.bytes()),
        wanted.units().saturating_sub(old.units()),
    );
    if extra != custody::Usage::ZERO {
        debit.grow(extra, &mut admit)?;
    }
    let excess = debit.usage().checked_sub(wanted)?;
    if excess != custody::Usage::ZERO {
        drop(debit.split(excess)?);
    }
    Ok(())
}

impl Draft {
    fn relation_counts(
        &self,
        field: RelationField,
        old: usize,
        new: usize,
    ) -> Result<Counts, DomError> {
        let mut counts = self.counts;
        let target = match field {
            RelationField::ManualNodes | RelationField::AssignedNodes => {
                Some(&mut counts.slot_references)
            }
            RelationField::AutofocusCandidates => Some(&mut counts.autofocus_candidates),
            _ => None,
        };
        if let Some(target) = target {
            *target = target
                .checked_sub(old)
                .and_then(|n| n.checked_add(new))
                .ok_or(DomError::RepresentationExhausted)?;
        }
        self.admit_node_counts(counts, self.nodes, self.units, self.dom_limits)?;
        Ok(counts)
    }
    fn private_relation_counts(
        &self,
        key: Key,
        field: RelationField,
        old: usize,
        new: usize,
    ) -> Result<Counts, DomError> {
        if !self.is_private(self.live_key(key)) {
            return Ok(self.private_counts);
        }
        let mut before = Counts::default();
        let mut after = Counts::default();
        match field {
            RelationField::ManualNodes | RelationField::AssignedNodes => {
                before.slot_references = old;
                after.slot_references = new;
            }
            RelationField::AutofocusCandidates => {
                before.autofocus_candidates = old;
                after.autofocus_candidates = new;
            }
            _ => {}
        }
        self.private_counts.checked_replace(before, after)
    }
    pub(in crate::store) fn relations_insert(
        &mut self,
        key: Key,
        field: RelationField,
        index: usize,
        values: &[Key],
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let old = self.private_node_mut(key).relation_mut(field).len();
        let next = old
            .checked_add(values.len())
            .ok_or(DomError::RepresentationExhausted)?;
        let counts = self.relation_counts(field, old, next)?;
        let private_counts = self.private_relation_counts(key, field, old, next)?;
        let undo = self.prepare_undo_slots(
            2 + usize::from(matches!(field, RelationField::Children)),
            cancel,
        )?;
        let inbound =
            self.prepare_inbound_delta(key, std::iter::empty(), values.iter().copied(), cancel)?;
        #[cfg(test)]
        super::queues::admission_observation::before_relation(self, key, field);
        #[cfg(test)]
        let growth = if matches!(field, RelationField::Children)
            && (self.records_for(key).children.canonical_backing().is_some()
                || next > self.records_for(key).children.capacity())
        {
            source::formation::mutation_observation::before(
                self,
                source::formation::mutation_observation::Site::Children,
                cancel,
            )?
        } else {
            None
        };
        let slots = if matches!(field, RelationField::Children) {
            self.prepare_slot_children(key, index, 0, values, cancel)?
        } else {
            None
        };
        let formation = source::formation::Formation::new(self, cancel)?;
        let ledger = self.custody_ledger().clone();
        let inverse = self.private_node_mut(key).relation_mut(field).insert(
            index,
            values,
            undo,
            &ledger,
            |usage| match &formation {
                Some(b) => b.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
            || source::formation::step(formation.as_ref(), cancel),
        )?;
        #[cfg(test)]
        if matches!(field, RelationField::Children) {
            source::formation::mutation_observation::grew(
                growth,
                self.records_for(key).children.capacity() * size_of::<Key>(),
            );
        }
        self.commit_inbound(inbound);
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        let oldprivatecounts = std::mem::replace(&mut self.private_counts, private_counts);
        if let Some(inverse) = inverse {
            self.journal.push_prepared(Undo::Relations {
                key,
                field,
                inverse,
                counts: oldcounts,
                private_counts: oldprivatecounts,
            });
        }
        Ok(())
    }
    pub(in crate::store) fn relations_remove(
        &mut self,
        key: Key,
        field: RelationField,
        index: usize,
        count: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let old = self.private_node_mut(key).relation_mut(field).len();
        let next = old.checked_sub(count).ok_or(DomError::IndexSize)?;
        let counts = self.relation_counts(field, old, next)?;
        let private_counts = self.private_relation_counts(key, field, old, next)?;
        let undo = self.prepare_undo_slots(
            2 + usize::from(matches!(field, RelationField::Children)),
            cancel,
        )?;
        let view = self.relation_view(key, field);
        let end = index
            .checked_add(count)
            .filter(|end| *end <= view.len())
            .ok_or(DomError::IndexSize)?;
        let inbound = self.prepare_inbound_delta(
            key,
            (index..end).map(|i| view.get(i).expect("bounded removed relation")),
            std::iter::empty(),
            cancel,
        )?;
        let slots = if matches!(field, RelationField::Children) {
            self.prepare_slot_children(key, index, count, &[], cancel)?
        } else {
            None
        };
        let formation = source::formation::Formation::new(self, cancel)?;
        let ledger = self.custody_ledger().clone();
        let inverse = self.private_node_mut(key).relation_mut(field).remove(
            index,
            count,
            undo,
            &ledger,
            |usage| match &formation {
                Some(b) => b.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
            || source::formation::step(formation.as_ref(), cancel),
        )?;
        self.commit_inbound(inbound);
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        let oldprivatecounts = std::mem::replace(&mut self.private_counts, private_counts);
        if let Some(inverse) = inverse {
            self.journal.push_prepared(Undo::Relations {
                key,
                field,
                inverse,
                counts: oldcounts,
                private_counts: oldprivatecounts,
            });
        }
        Ok(())
    }
    pub(in crate::store) fn relations_replace(
        &mut self,
        key: Key,
        field: RelationField,
        values: Vec<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let old = self.private_node_mut(key).relation_mut(field).len();
        let counts = self.relation_counts(field, old, values.len())?;
        let private_counts = self.private_relation_counts(key, field, old, values.len())?;
        let undo = self.prepare_undo_slots(
            2 + usize::from(matches!(field, RelationField::Children)),
            cancel,
        )?;
        let inbound = self.prepare_inbound_delta(
            key,
            self.relation_view(key, field).iter(),
            values.iter().copied(),
            cancel,
        )?;
        let slots = if matches!(field, RelationField::Children) {
            self.prepare_slot_children_replace(key, &values, cancel)?
        } else {
            None
        };
        let formation = source::formation::Formation::new(self, cancel)?;
        let ledger = self.custody_ledger().clone();
        let receipt = custody::Usage::new(
            values
                .capacity()
                .checked_mul(size_of::<Key>())
                .ok_or(DomError::RepresentationExhausted)?,
            0,
        );
        let debit = ledger.retain(receipt, |usage| self.admit_custody_usage(usage, cancel))?;
        let replacement = Sequence::staged(values, debit, Key::Existing);
        let inverse = self
            .private_node_mut(key)
            .relation_mut(field)
            .replace(replacement, undo);
        self.commit_inbound(inbound);
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        let oldprivatecounts = std::mem::replace(&mut self.private_counts, private_counts);
        if let Some(inverse) = inverse {
            self.journal.push_prepared(Undo::Relations {
                key,
                field,
                inverse,
                counts: oldcounts,
                private_counts: oldprivatecounts,
            });
        }
        drop(formation);
        Ok(())
    }
    pub(in crate::store) fn relations_retain(
        &mut self,
        key: Key,
        field: RelationField,
        mut keep: impl FnMut(Key) -> bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut values = Vec::new();
        let source = self.relation_view(key, field);
        source::formation::reserve(&mut values, source.len(), formation.as_ref(), cancel)?;
        #[cfg(test)]
        let mut copied = super::sequence::CopyObservation::relation();
        for value in source.iter() {
            self.source_step(cancel)?;
            if keep(value) {
                values.push(value);
                #[cfg(test)]
                copied.pushed::<Key>();
            }
        }
        if values.len() == source.len() {
            return Ok(());
        }
        self.relations_replace(key, field, values, cancel)
    }
    fn relation_view(
        &self,
        key: Key,
        field: RelationField,
    ) -> super::SequenceView<'_, NodeId, Key> {
        let view = self.records_for(key).relation(field).view();
        match &self.phase_map {
            Some(map) => view.aliases(&map.ids, source::alias_key),
            None => view,
        }
    }
    pub(in crate::store) fn records_for(&self, key: Key) -> &DraftNode {
        let key = self.live_key(key);
        match key {
            Key::Existing(id) => {
                self.records[*self.existing.get(id.slot).expect("materialized index")].as_ref()
            }
            Key::Created(index) => self.created[index].as_ref(),
        }
        .expect("live private record")
    }
    pub(in crate::store) fn fullscreen_events_push(
        &mut self,
        key: Key,
        event: crate::DomFullscreenEvent,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let len = self.records_for(key).fullscreen_events.len();
        let mut counts = self.counts;
        counts.fullscreen_events = counts
            .fullscreen_events
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        self.admit_node_counts(counts, self.nodes, self.units, self.dom_limits)?;
        let mut private_counts = self.private_counts;
        if self.is_private(self.live_key(key)) {
            private_counts.fullscreen_events = private_counts
                .fullscreen_events
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let undo = self.prepare_undo(cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let ledger = self.custody_ledger().clone();
        let inverse = self.private_node_mut(key).fullscreen_events.insert(
            len,
            &[event],
            undo,
            &ledger,
            |usage| match &formation {
                Some(b) => b.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
            || source::formation::step(formation.as_ref(), cancel),
        )?;
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        let oldprivatecounts = std::mem::replace(&mut self.private_counts, private_counts);
        if let Some(inverse) = inverse {
            self.journal.push_prepared(Undo::FullscreenEvents {
                key,
                inverse,
                counts: oldcounts,
                private_counts: oldprivatecounts,
            });
        }
        Ok(())
    }
    pub(in crate::store) fn fullscreen_events_clear(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let len = self.records_for(key).fullscreen_events.len();
        let undo = self.prepare_undo(cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let ledger = self.custody_ledger().clone();
        let inverse = self.private_node_mut(key).fullscreen_events.remove(
            0,
            len,
            undo,
            &ledger,
            |usage| match &formation {
                Some(b) => b.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
            || source::formation::step(formation.as_ref(), cancel),
        )?;
        let oldcounts = self.counts;
        let oldprivatecounts = self.private_counts;
        if self.is_private(self.live_key(key)) {
            self.private_counts.fullscreen_events -= len;
        }
        self.counts.fullscreen_events -= len;
        if let Some(inverse) = inverse {
            self.journal.push_prepared(Undo::FullscreenEvents {
                key,
                inverse,
                counts: oldcounts,
                private_counts: oldprivatecounts,
            });
        }
        Ok(())
    }
}
impl DraftNode {
    fn relation(&self, field: RelationField) -> &Sequence<NodeId, Key> {
        match field {
            RelationField::Children => &self.children,
            RelationField::ManualNodes => &self.manual_nodes,
            RelationField::AssignedNodes => &self.assigned_nodes,
            RelationField::RenderBlocking => &self.render_blocking,
            RelationField::AutofocusCandidates => &self.autofocus_candidates,
            RelationField::OpenDialogs => &self.open_dialogs,
            RelationField::TopLayer => &self.top_layer,
            RelationField::PendingTopLayerRemovals => &self.pending_top_layer_removals,
        }
    }
}

pub(in crate::store) struct RetiredNode {
    pub style: bool,
    pub document: Key,
}
impl NodeFields {
    pub(in crate::store) fn logical_units(&self) -> Result<usize, DomError> {
        let mut units = kind_units(&self.kind)?;
        for n in [
            self.base.as_ref().map_or(0, crate::base::BaseState::units),
            self.hyperlink
                .as_ref()
                .map_or(0, crate::hyperlink::HyperlinkState::units),
            self.encoding_url_document
                .as_ref()
                .map_or(0, crate::hyperlink::EncodingUrlDocument::units),
            self.base_document
                .as_ref()
                .map_or(0, crate::base::BaseDocument::units),
            self.form_past_names
                .as_ref()
                .map_or(0, super::Names::logical_units),
        ] {
            units = units
                .checked_add(n)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(units)
    }
}
impl Draft {
    pub(in crate::store) fn retire_node(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<RetiredNode, DomError> {
        let key = self.live_key(key);
        self.materialize_node(key, cancel)?;
        let counts = self
            .counts
            .checked_replace(Counts::node(self.node(key)?)?, Counts::default())?;
        let units = self.records_for(key).fields.value().logical_units()?;
        let style =
            matches!(self.node(key)?.kind,DomNodeKind::Element(e) if e.style_state().is_some());
        let document = self.node(key)?.document;
        let undo = self.prepare_undo_slots(5, cancel)?;
        let inbound = self.prepare_inbound_node(key, self.node(key)?, false, cancel)?;
        let controls = self.prepare_control_retired(key, cancel)?;
        let slots = self.prepare_slot_retired(key, cancel)?;
        let private = self.is_private(key);
        let next_private_counts = if private {
            self.private_counts
                .checked_replace(Counts::node(self.node(key)?)?, Counts::default())?
        } else {
            self.private_counts
        };
        let active = self.node(key)?.active_range;
        if active != crate::DomActiveRangeState::Unavailable {
            self.update_active_range_dependency(
                key,
                active,
                crate::DomActiveRangeState::Unavailable,
                cancel,
            )?;
        }
        let oldprivatecounts = self.private_counts;
        let oldprivatenodes = self.private_nodes;
        let oldprivateunits = self.private_units;
        if private {
            self.private_counts = next_private_counts;
            self.private_nodes -= 1;
            self.private_units -= units;
        }
        self.commit_inbound(inbound);
        if let Some(plan) = controls {
            self.commit_control_plan(plan);
        }
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        let old = match key {
            Key::Existing(id) => self.records[*self
                .existing
                .get(id.slot)
                .expect("materialized retiring record")]
            .take(),
            Key::Created(index) => self.created[index].take(),
        }
        .expect("live retiring record");
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        let oldnodes = self.nodes;
        let oldunits = self.units;
        self.nodes -= 1;
        self.units = self
            .units
            .checked_sub(units)
            .expect("canonical logical units cover retired record");
        if undo {
            self.journal.push_prepared(Undo::Retired {
                key,
                old,
                counts: oldcounts,
                nodes: oldnodes,
                units: oldunits,
                private_nodes: oldprivatenodes,
                private_units: oldprivateunits,
                private_counts: oldprivatecounts,
            });
        }
        Ok(RetiredNode { style, document })
    }
}

impl DraftNode {
    pub(in crate::store) fn fresh(
        mut fields: NodeFields,
        mut incoming: custody::Debit,
        ledger: &custody::Ledger,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut staged = |values: Vec<Key>| -> Result<Sequence<NodeId, Key>, DomError> {
            let bytes = values
                .capacity()
                .checked_mul(size_of::<Key>())
                .ok_or(DomError::RepresentationExhausted)?;
            let debit = ledger.retain(custody::Usage::new(bytes, 0), &mut admit)?;
            Ok(Sequence::staged(values, debit, Key::Existing))
        };
        let render = fields
            .base_document
            .as_mut()
            .map(|v| std::mem::take(&mut v.render_blocking))
            .unwrap_or_default();
        let candidates = match fields.autofocus_document.as_mut() {
            Some(crate::autofocus::DocumentAutofocusState::Known { candidates, .. }) => {
                std::mem::take(candidates)
            }
            _ => Vec::new(),
        };
        let (open, top, pending) = match fields.interactive_document.as_mut() {
            Some(crate::dialog::DocumentInteractiveState::Known {
                open_dialogs,
                top_layer,
                pending_top_layer_removals,
            }) => (
                std::mem::take(open_dialogs),
                std::mem::take(top_layer),
                std::mem::take(pending_top_layer_removals),
            ),
            _ => (Vec::new(), Vec::new(), Vec::new()),
        };
        let children = staged(Vec::new())?;
        let manual_nodes = staged(Vec::new())?;
        let assigned_nodes = staged(Vec::new())?;
        let render_blocking = staged(render)?;
        let autofocus_candidates = staged(candidates)?;
        let open_dialogs = staged(open)?;
        let top_layer = staged(top)?;
        let pending_top_layer_removals = staged(pending)?;
        resize_debit(&mut incoming, fields.usage(|| Ok(()))?, &mut admit)?;
        let fields_debit = incoming;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::RecordMaterialized {
                bytes: size_of::<NodeFields>(),
            });
        }
        let fullscreen_events = Sequence::empty(std::convert::identity, ledger, &mut admit)?;
        Ok(Self {
            fields: custody::Held::new(Box::new(fields), fields_debit),
            children,
            manual_nodes,
            assigned_nodes,
            render_blocking,
            autofocus_candidates,
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
            fullscreen_events,
        })
    }
}
impl Draft {
    pub(in crate::store) fn reserve_created_arena(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.created.len() < self.created.capacity() {
            return Ok(());
        }
        let undo = self.prepare_undo(cancel)?;
        let wanted = self
            .created
            .len()
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?
            .max(self.created.capacity().saturating_mul(2));
        let ledger = self.custody_ledger().clone();
        let bytes = wanted
            .checked_mul(size_of::<Option<DraftNode>>())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut debit = ledger.retain(custody::Usage::new(bytes, 0), |usage| {
            self.admit_custody_usage(usage, cancel)
        })?;
        let mut next = Vec::new();
        next.try_reserve_exact(wanted)
            .map_err(|_| DomError::AllocationFailed)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: next.capacity() * size_of::<Option<DraftNode>>(),
            });
        }
        resize_debit(
            &mut debit,
            custody::Usage::new(
                next.capacity()
                    .checked_mul(size_of::<Option<DraftNode>>())
                    .ok_or(DomError::RepresentationExhausted)?,
                0,
            ),
            |usage| self.admit_custody_usage(usage, cancel),
        )?;
        for _ in &self.created {
            self.source_step(cancel)?;
        }
        #[cfg(test)]
        let moved_bytes = self.created.len() * size_of::<Option<DraftNode>>();
        next.append(&mut self.created);
        #[cfg(test)]
        super::sequence::record_copied_bytes(moved_bytes);
        let old = std::mem::replace(&mut self.created, next);
        let debit = std::mem::replace(&mut self.created_charge, debit);
        if undo {
            self.journal
                .push_prepared(Undo::CreatedCapacity { old, debit });
        } else {
            drop(old);
            drop(debit);
        }
        Ok(())
    }
}

impl Draft {
    pub(in crate::store) fn set_created_dirty(
        &mut self,
        index: usize,
        value: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.created_dirty.get(index).copied().unwrap_or(false) == value {
            return Ok(());
        }
        let undo = self.prepare_undo(cancel)?;
        let (next, custody) = if value {
            let record = bank::ControlledRecord::new(
                true,
                custody::Usage::new(size_of::<bool>(), 0),
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
            )?;
            self.created_dirty.set_controlled(
                &self.created_dirty_custody,
                index,
                record,
                false,
                bank::Totals {
                    bytes: size_of::<bool>(),
                    units: 0,
                    occurrences: 1,
                },
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
            )?
        } else {
            let (next, custody, _) = self.created_dirty.remove_controlled(
                &self.created_dirty_custody,
                index,
                self.custody_registry(),
                |usage| self.admit_custody_usage(usage, cancel),
            )?;
            (next, custody)
        };
        let before = std::mem::replace(&mut self.created_dirty, next);
        let custody = std::mem::replace(&mut self.created_dirty_custody, custody);
        if undo {
            self.journal
                .push_prepared(Undo::CreatedDirty { before, custody });
        }
        Ok(())
    }
}

impl Draft {
    pub(in crate::store) fn relations_replace_owned(
        &mut self,
        key: Key,
        field: RelationField,
        owned: source::scratch::OwnedScratch<Key>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.materialize_node(key, cancel)?;
        let old = self.records_for(key).relation(field).len();
        let counts = self.relation_counts(field, old, owned.values().len())?;
        let private_counts = self.private_relation_counts(key, field, old, owned.values().len())?;
        let undo = self.prepare_undo_slots(
            2 + usize::from(matches!(field, RelationField::Children)),
            cancel,
        )?;
        let inbound = self.prepare_inbound_delta(
            key,
            self.relation_view(key, field).iter(),
            owned.values().iter().copied(),
            cancel,
        )?;
        let slots = if matches!(field, RelationField::Children) {
            self.prepare_slot_children_replace(key, owned.values(), cancel)?
        } else {
            None
        };
        let ledger = self.custody_ledger().clone();
        let values = owned.into_held(&ledger, |usage| self.admit_custody_usage(usage, cancel))?;
        let (values, debit) = values.into_parts();
        let inverse = self
            .private_node_mut(key)
            .relation_mut(field)
            .replace(Sequence::staged(values, debit, Key::Existing), undo);
        self.commit_inbound(inbound);
        if let Some(plan) = slots {
            self.commit_slot_plan(plan);
        }
        let oldcounts = std::mem::replace(&mut self.counts, counts);
        let oldprivatecounts = std::mem::replace(&mut self.private_counts, private_counts);
        if let Some(inverse) = inverse {
            self.journal.push_prepared(Undo::Relations {
                key,
                field,
                inverse,
                counts: oldcounts,
                private_counts: oldprivatecounts,
            });
        }
        Ok(())
    }
}

impl Draft {
    pub(in crate::store) fn prepare_output_buffer<T: Copy>(
        &self,
        values: &Vec<T>,
        additional: usize,
        cancel: &AtomicBool,
    ) -> Result<Option<custody::Held<Vec<T>>>, DomError> {
        let needed = values
            .len()
            .checked_add(additional)
            .ok_or(DomError::RepresentationExhausted)?;
        if needed <= values.capacity() {
            return Ok(None);
        }
        let capacity = needed
            .max(
                values
                    .capacity()
                    .checked_mul(2)
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .max(4);
        let usage = custody::Usage::new(
            capacity
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?,
            0,
        );
        let mut debit = self
            .custody_ledger()
            .retain(usage, |usage| self.admit_custody_usage(usage, cancel))?;
        let mut next = Vec::new();
        next.try_reserve_exact(capacity)
            .map_err(|_| DomError::AllocationFailed)?;
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                bytes: next.capacity() * size_of::<T>(),
            });
        }
        resize_debit(
            &mut debit,
            custody::Usage::new(
                next.capacity()
                    .checked_mul(size_of::<T>())
                    .ok_or(DomError::RepresentationExhausted)?,
                0,
            ),
            |usage| self.admit_custody_usage(usage, cancel),
        )?;
        #[cfg(test)]
        let mut copied = super::sequence::CopyObservation::default();
        for value in values {
            self.source_step(cancel)?;
            next.push(*value);
            #[cfg(test)]
            copied.pushed::<T>();
        }
        Ok(Some(custody::Held::new(next, debit)))
    }
    pub(in crate::store) fn record_dispatch_retirement(
        &mut self,
        target: NodeId,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let undo = self.prepare_undo_slots(2, cancel)?;
        if let Some(next) = self.prepare_output_buffer(&self.dispatch_retirements, 1, cancel)? {
            let (values, debit) = next.into_parts();
            let old = std::mem::replace(&mut self.dispatch_retirements, values);
            let debit = std::mem::replace(&mut self.dispatch_retirements_charge, debit);
            let before = custody::Held::new(old, debit);
            if undo {
                self.journal
                    .push_prepared(Undo::RetirementsCapacity(before));
            }
        }
        let len = self.dispatch_retirements.len();
        self.source_step(cancel)?;
        self.dispatch_retirements.push((target, document));
        #[cfg(test)]
        super::sequence::record_copied_bytes(size_of::<(NodeId, Key)>());
        if undo {
            self.journal.push_prepared(Undo::RetirementsLength(len));
        }
        Ok(())
    }
}

#[cfg(test)]
impl DraftNode {
    pub(super) fn journal_bytes(&self) -> usize {
        let mut bytes = self.fields.debit().usage().bytes();
        for seq in [
            &self.children,
            &self.manual_nodes,
            &self.assigned_nodes,
            &self.render_blocking,
            &self.autofocus_candidates,
            &self.open_dialogs,
            &self.top_layer,
            &self.pending_top_layer_removals,
        ] {
            if seq.canonical_backing().is_none() {
                bytes = bytes
                    .checked_add(seq.receipt().bytes())
                    .expect("admitted saved record backing");
            }
        }
        if self.fullscreen_events.canonical_backing().is_none() {
            bytes = bytes
                .checked_add(self.fullscreen_events.receipt().bytes())
                .expect("admitted saved event backing");
        }
        bytes
    }
}

#[cfg(test)]
mod movement_metrics_tests {
    use super::*;
    use crate::store::scale_metrics::{self, Counters, Hooks, Measurement};

    fn measured<R>(run: impl FnOnce() -> R) -> (R, Counters) {
        let measurement = Measurement::begin().unwrap();
        let result = run();
        (result, measurement.finish(Hooks::default()).unwrap())
    }

    fn draft(source: bool) -> (Draft, Key) {
        let cancel = AtomicBool::new(false);
        let limits = DomLimits::new(16, 8192, 32, 0, 0, 8, 1_000_000).unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let commit = dom.apply(batch, &cancel).unwrap();
        let document = Key::Existing(commit.document(&document).unwrap().node());
        let draft = if source {
            Draft::new_source_root(
                &dom,
                Arc::new(BatchIdentity),
                &[],
                0,
                crate::DomSourceLimits::new(16_000_000, 1_000_000, 1_000_000, 8, 16, 16, 1),
                &cancel,
            )
            .unwrap()
            .0
        } else {
            Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap()
        };
        (draft, document)
    }

    #[test]
    fn raw_reserve_reports_actual_growth_and_only_a_relocated_initialized_prefix() {
        if !scale_metrics::ENABLED {
            return;
        }
        let mut values = vec![11u64, 22];
        let before = (values.capacity(), values.as_ptr(), values.len());
        let (result, counters) = measured(|| reserve(&mut values, before.0 + 1));
        result.unwrap();
        assert_eq!(values, [11, 22]);
        assert!(values.capacity() > before.0);
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            values.capacity() * size_of::<u64>()
        );
        assert_eq!(
            counters.sequence_materialized_bytes,
            if values.as_ptr() == before.1 {
                0
            } else {
                before.2 * size_of::<u64>()
            }
        );
        assert_eq!(counters.relation_sequence_materialized_bytes, 0);

        let (result, counters) = measured(|| reserve(&mut values, 0));
        result.unwrap();
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);

        let before = (values.capacity(), values.as_ptr());
        let (result, counters) = measured(|| reserve(&mut values, usize::MAX));
        assert_eq!(result, Err(DomError::AllocationFailed));
        assert_eq!((values.capacity(), values.as_ptr()), before);
        assert_eq!(values, [11, 22]);
        assert_eq!(counters.structural_allocations, 0);
        assert_eq!(counters.sequence_materialized_bytes, 0);
    }

    #[test]
    fn real_arena_and_ticket_appends_report_each_initialized_element() {
        if !scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        for source in [false, true] {
            let (mut draft, document) = draft(source);
            draft.reserve_record_arena(&cancel).unwrap();
            let (result, counters) = measured(|| draft.materialize_node(document, &cancel));
            result.unwrap();
            assert_eq!(draft.records.len(), 1);
            assert_eq!(
                counters.sequence_materialized_bytes,
                size_of::<Option<DraftNode>>()
            );
            assert_eq!(counters.relation_sequence_materialized_bytes, 0);

            draft.reserve_created_arena(&cancel).unwrap();
            let (result, counters) = measured(|| {
                draft.allocate(
                    DomNodeKind::Comment("new".into()),
                    document,
                    draft.dom_limits,
                    &cancel,
                )
            });
            let created = result.unwrap();
            assert_eq!(draft.created.len(), 1);
            assert_eq!(
                counters.sequence_materialized_bytes,
                size_of::<Option<DraftNode>>()
            );
            assert_eq!(counters.relation_sequence_materialized_bytes, 0);

            let (result, counters) = measured(|| draft.register_ticket(created, &cancel));
            result.unwrap();
            assert_eq!(draft.tickets, [created]);
            assert_eq!(counters.sequence_materialized_bytes, size_of::<Key>());
            assert_eq!(counters.relation_sequence_materialized_bytes, 0);

            let Key::Existing(target) = document else {
                unreachable!()
            };
            let (result, counters) =
                measured(|| draft.record_dispatch_retirement(target, document, &cancel));
            result.unwrap();
            assert_eq!(draft.dispatch_retirements, [(target, document)]);
            assert_eq!(
                counters.sequence_materialized_bytes,
                size_of::<(NodeId, Key)>()
            );
            assert_eq!(counters.relation_sequence_materialized_bytes, 0);
        }
    }

    #[test]
    fn real_ordinary_and_unmarked_source_arenas_count_only_completed_backing_moves() {
        if !scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        for source in [false, true] {
            let (mut draft, document) = draft(source);
            draft.materialize_node(document, &cancel).unwrap();
            draft
                .allocate(
                    DomNodeKind::Comment("new".into()),
                    document,
                    draft.dom_limits,
                    &cancel,
                )
                .unwrap();
            assert_eq!(draft.journal.latest_mark_id(), None);
            assert_eq!(draft.records.len(), draft.records.capacity());
            assert_eq!(draft.created.len(), draft.created.capacity());
            let old_records = draft.records.len();
            let (result, counters) = measured(|| draft.reserve_record_arena(&cancel));
            result.unwrap();
            assert_eq!(counters.structural_allocations, 1);
            assert_eq!(
                counters.structural_bytes,
                draft.records.capacity() * size_of::<Option<DraftNode>>()
            );
            assert_eq!(
                counters.sequence_materialized_bytes,
                old_records * size_of::<Option<DraftNode>>()
            );
            let old_created = draft.created.len();
            let (result, counters) = measured(|| draft.reserve_created_arena(&cancel));
            result.unwrap();
            assert_eq!(counters.structural_allocations, 1);
            assert_eq!(
                counters.structural_bytes,
                draft.created.capacity() * size_of::<Option<DraftNode>>()
            );
            assert_eq!(
                counters.sequence_materialized_bytes,
                old_created * size_of::<Option<DraftNode>>()
            );
            let (result, counters) = measured(|| draft.reserve_created_arena(&cancel));
            result.unwrap();
            assert_eq!(counters.structural_allocations, 0);
            assert_eq!(counters.sequence_materialized_bytes, 0);
        }
    }

    #[test]
    fn arena_work_refusal_counts_reserved_capacity_without_unperformed_moves() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (mut draft, document) = draft(true);
        let cancel = AtomicBool::new(false);
        draft.materialize_node(document, &cancel).unwrap();
        assert_eq!(draft.records.len(), draft.records.capacity());
        let pointer = draft.records.as_ptr();
        let capacity = draft.records.capacity();
        let before = draft.custody_ledger().total();
        let work = draft.source_work.load(Ordering::Relaxed) + 1;
        draft.source_limits = Some(crate::DomSourceLimits::new(
            16_000_000, 1_000_000, work, 8, 16, 16, 1,
        ));
        let (result, counters) = measured(|| draft.reserve_record_arena(&cancel));
        assert_eq!(result, Err(DomError::SourceWorkLimit { limit: work }));
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            2 * capacity * size_of::<Option<DraftNode>>()
        );
        assert_eq!(counters.sequence_materialized_bytes, 0);
        assert_eq!(draft.records.as_ptr(), pointer);
        assert_eq!(draft.records.len(), 1);
        assert_eq!(draft.custody_ledger().total(), before);
    }

    #[test]
    fn output_growth_refusal_reports_only_its_completed_copy_prefix() {
        if !scale_metrics::ENABLED {
            return;
        }
        let (mut draft, _) = draft(true);
        let cancel = AtomicBool::new(false);
        let values = vec![11_usize, 22, 33];
        let before = draft.custody_ledger().total();
        let work = draft.source_work.load(Ordering::Relaxed) + 3;
        draft.source_limits = Some(crate::DomSourceLimits::new(
            16_000_000, 1_000_000, work, 8, 16, 16, 1,
        ));
        let (result, counters) =
            measured(|| draft.prepare_output_buffer(&values, values.capacity(), &cancel));
        assert!(matches!(result, Err(DomError::SourceWorkLimit { limit }) if limit == work));
        assert_eq!(counters.structural_allocations, 1);
        assert_eq!(
            counters.structural_bytes,
            2 * values.capacity() * size_of::<usize>()
        );
        assert_eq!(counters.sequence_materialized_bytes, 2 * size_of::<usize>());
        assert_eq!(values, [11, 22, 33]);
        assert_eq!(draft.custody_ledger().total(), before);
    }
}
