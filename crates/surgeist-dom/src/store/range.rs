//! The one live Range bank. Node references follow the Draft's ordinary mapping.
use super::transaction::KindRead;
use super::*;
use crate::range::{Boundary, Record};
use crate::{DomActiveRangeState, DomRangeId, DomRangeOperand};
use std::cmp::Ordering as Position;

#[cfg(test)]
mod admission_tests;
#[cfg(test)]
mod allocation_tests;
#[cfg(test)]
mod position_tests;

#[cfg(test)]
mod allocation_observation {
    use std::cell::Cell;

    thread_local! {
        static ENABLED: Cell<bool> = const { Cell::new(false) };
        static VISITS: Cell<usize> = const { Cell::new(0) };
    }

    pub(super) struct Observation;

    pub(super) fn start() -> Observation {
        VISITS.set(0);
        ENABLED.set(true);
        Observation
    }

    pub(super) fn visit() {
        if ENABLED.get() {
            VISITS.set(VISITS.get() + 1);
        }
    }

    impl Observation {
        pub(super) fn finish(self) -> usize {
            ENABLED.set(false);
            VISITS.get()
        }
    }

    impl Drop for Observation {
        fn drop(&mut self) {
            ENABLED.set(false);
        }
    }
}

#[cfg(test)]
mod maintenance_observation {
    use std::cell::RefCell;
    thread_local! { static AFTER_WRITE: RefCell<Option<Box<dyn FnMut()>>> = const { RefCell::new(None) }; }
    pub(super) struct Observation;
    pub(super) fn after_write(callback: impl FnMut() + 'static) -> Observation {
        AFTER_WRITE.with(|slot| {
            assert!(slot.borrow().is_none());
            *slot.borrow_mut() = Some(Box::new(callback));
        });
        Observation
    }
    pub(super) fn wrote() {
        if let Some(mut callback) = AFTER_WRITE.with(|slot| slot.borrow_mut().take()) {
            callback();
            AFTER_WRITE.with(|slot| *slot.borrow_mut() = Some(callback));
        }
    }
    impl Drop for Observation {
        fn drop(&mut self) {
            AFTER_WRITE.with(|slot| *slot.borrow_mut() = None);
        }
    }
}

impl Draft {
    pub(super) fn range_record(&self, id: DomRangeId) -> Result<Record<Key>, DomError> {
        if id.owner != self.store {
            return Err(DomError::ForeignOwner);
        }
        self.ranges
            .slot(id.slot)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.record)
            .map(|record| record.map(|key| self.live_key(key)))
            .ok_or(DomError::StaleRange)
    }
    pub(super) fn resolve_range(&self, operand: &DomRangeOperand) -> Result<DomRangeId, DomError> {
        let id = match operand {
            DomRangeOperand::Existing(id) => *id,
            DomRangeOperand::Created(ticket) => {
                if !Arc::ptr_eq(&ticket.owner, &self.owner) {
                    return Err(DomError::ForeignBatch);
                }
                *self
                    .range_tickets
                    .get(ticket.index)
                    .ok_or(DomError::UnavailableCreation)?
            }
        };
        self.range_record(id)?;
        Ok(id)
    }
    fn range_node_length(&self, key: Key) -> Result<usize, DomError> {
        let node = self.node(key)?;
        Ok(match node.kind() {
            KindRead::DocumentType(_) => 0,
            KindRead::Text(data) | KindRead::Comment(data) => data.len(),
            KindRead::ProcessingInstruction(data) => data.data().len(),
            _ => node.children().len(),
        })
    }
    fn range_root(&self, mut key: Key, cancel: &AtomicBool) -> Result<Key, DomError> {
        loop {
            self.source_step(cancel)?;
            match self.node(key)?.parent() {
                Some(parent) => key = parent,
                None => return Ok(key),
            }
        }
    }
    fn range_inclusive_ancestor(
        &self,
        ancestor: Key,
        mut node: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        loop {
            self.source_step(cancel)?;
            if node == ancestor {
                return Ok(true);
            }
            match self.node(node)?.parent() {
                Some(parent) => node = parent,
                None => return Ok(false),
            }
        }
    }
    fn range_child_under(
        &self,
        ancestor: Key,
        mut child: Key,
        cancel: &AtomicBool,
    ) -> Result<Key, DomError> {
        loop {
            self.source_step(cancel)?;
            let parent = self
                .node(child)?
                .parent()
                .ok_or(DomError::HierarchyRequest)?;
            if parent == ancestor {
                return Ok(child);
            }
            child = parent;
        }
    }
    fn range_child_index(
        &self,
        parent: Key,
        child: Key,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        let parent = self.live_key(parent);
        let child = self.live_key(child);
        let children = self.node(parent)?.children();
        self.source_step(cancel)?;
        self.slot_index
            .position(child)
            .filter(|position| {
                position.parent == parent && children.get(position.ordinal) == Some(child)
            })
            .map(|position| position.ordinal)
            .ok_or(DomError::HierarchyRequest)
    }
    fn range_position(
        &self,
        a: Boundary<Key>,
        b: Boundary<Key>,
        cancel: &AtomicBool,
    ) -> Result<Option<Position>, DomError> {
        if a.container == b.container {
            return Ok(Some(a.offset.cmp(&b.offset)));
        }
        if self.range_root(a.container, cancel)? != self.range_root(b.container, cancel)? {
            return Ok(None);
        }
        if self.range_inclusive_ancestor(a.container, b.container, cancel)? {
            let child = self.range_child_under(a.container, b.container, cancel)?;
            let index = self.range_child_index(a.container, child, cancel)?;
            return Ok(Some(if a.offset <= index {
                Position::Less
            } else {
                Position::Greater
            }));
        }
        if self.range_inclusive_ancestor(b.container, a.container, cancel)? {
            return self
                .range_position(b, a, cancel)
                .map(|position| position.map(Position::reverse));
        }
        let mut child = a.container;
        loop {
            self.source_step(cancel)?;
            let parent = self
                .node(child)?
                .parent()
                .ok_or(DomError::HierarchyRequest)?;
            if self.range_inclusive_ancestor(parent, b.container, cancel)? {
                let other = self.range_child_under(parent, b.container, cancel)?;
                return Ok(Some(
                    self.range_child_index(parent, child, cancel)?
                        .cmp(&self.range_child_index(parent, other, cancel)?),
                ));
            }
            child = parent;
        }
    }
    /// Independent publication oracle: locate children in raw semantic order,
    /// without consulting the maintained child-position index.
    #[cfg(test)]
    fn raw_range_child_index(
        &self,
        parent: Key,
        child: Key,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        for (index, candidate) in self.node(parent)?.children().iter().enumerate() {
            self.source_step(cancel)?;
            if candidate == child {
                return Ok(index);
            }
        }
        Err(DomError::HierarchyRequest)
    }
    #[cfg(test)]
    fn raw_range_position(
        &self,
        a: Boundary<Key>,
        b: Boundary<Key>,
        cancel: &AtomicBool,
    ) -> Result<Option<Position>, DomError> {
        if a.container == b.container {
            return Ok(Some(a.offset.cmp(&b.offset)));
        }
        if self.range_root(a.container, cancel)? != self.range_root(b.container, cancel)? {
            return Ok(None);
        }
        if self.range_inclusive_ancestor(a.container, b.container, cancel)? {
            let child = self.range_child_under(a.container, b.container, cancel)?;
            let index = self.raw_range_child_index(a.container, child, cancel)?;
            return Ok(Some(if a.offset <= index {
                Position::Less
            } else {
                Position::Greater
            }));
        }
        if self.range_inclusive_ancestor(b.container, a.container, cancel)? {
            return self
                .raw_range_position(b, a, cancel)
                .map(|position| position.map(Position::reverse));
        }
        let mut child = a.container;
        loop {
            self.source_step(cancel)?;
            let parent = self
                .node(child)?
                .parent()
                .ok_or(DomError::HierarchyRequest)?;
            if self.range_inclusive_ancestor(parent, b.container, cancel)? {
                let other = self.range_child_under(parent, b.container, cancel)?;
                return Ok(Some(
                    self.raw_range_child_index(parent, child, cancel)?
                        .cmp(&self.raw_range_child_index(parent, other, cancel)?),
                ));
            }
            child = parent;
        }
    }
    pub(super) fn create_range(
        &mut self,
        document: Key,
        ticket: usize,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(self.node(document)?.kind(), KindRead::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if ticket != self.range_tickets.len() {
            return Err(DomError::UnavailableCreation);
        }
        let available = self.ranges.lowest_vacant();
        let (index, generation) = if let Some(index) = available {
            self.source_step(cancel)?;
            #[cfg(test)]
            allocation_observation::visit();
            let slot = self
                .ranges
                .slot(index)
                .expect("derived availability identifies a Range slot");
            debug_assert!(slot.record.is_none() && !slot.retired && slot.generation < u64::MAX);
            (
                index,
                slot.generation
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?,
            )
        } else {
            admit(
                self.ranges
                    .len()
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?,
                limits.ranges,
                DomResource::Ranges,
            )?;
            (self.ranges.len(), 1)
        };
        self.reserve_range_ticket(cancel)?;
        self.source_admit(0, cancel)?;
        let boundary = Boundary {
            container: document,
            offset: 0,
        };
        self.write_range(
            index,
            crate::range::Slot {
                generation,
                retired: false,
                record: Some(Record {
                    start: boundary,
                    end: boundary,
                }),
            },
            cancel,
        )?;
        self.range_tickets.push(DomRangeId {
            owner: self.store,
            slot: index,
            generation,
        });
        Ok(())
    }
    pub(super) fn set_range_boundary(
        &mut self,
        id: DomRangeId,
        container: Key,
        offset: usize,
        start: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if matches!(self.node(container)?.kind(), KindRead::DocumentType(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if offset > self.range_node_length(container)? {
            return Err(DomError::IndexSize);
        }
        let mut record = self.range_record(id)?;
        let point = Boundary { container, offset };
        if start {
            if !matches!(
                self.range_position(point, record.end, cancel)?,
                Some(Position::Less | Position::Equal)
            ) {
                record.end = point;
            }
            record.start = point;
        } else {
            if !matches!(
                self.range_position(point, record.start, cancel)?,
                Some(Position::Greater | Position::Equal)
            ) {
                record.start = point;
            }
            record.end = point;
        }
        self.replace_range_record(id, record, cancel)
    }
    pub(super) fn set_active_range(
        &mut self,
        document: Key,
        id: Option<DomRangeId>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !matches!(self.node(document)?.kind(), KindRead::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if self.node(document)?.active_range() == DomActiveRangeState::Known(id) {
            return Ok(());
        }
        if let Some(id) = id {
            let record = self.range_record(id)?;
            if self.range_root(record.start.container, cancel)? != document
                || self.range_root(record.end.container, cancel)? != document
            {
                return Err(DomError::RangeAssociationIneligible);
            }
        }
        let mut active = self.active_range_mut(document, cancel)?;
        *active = DomActiveRangeState::Known(id);
        active.commit(cancel)
    }
    pub(super) fn retire_range(
        &mut self,
        id: DomRangeId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.range_record(id)?;
        let documents = self.range_associated_documents(id, cancel)?;
        let mut slot = self.ranges.slot(id.slot).ok_or(DomError::StaleRange)?;
        slot.record = None;
        slot.retired |= id.generation == u64::MAX;
        self.write_range(id.slot, slot, cancel)?;
        for document in documents.value() {
            self.source_step(cancel)?;
            match self.node(*document) {
                Ok(node) if node.active_range() == DomActiveRangeState::Known(Some(id)) => {
                    let mut active = self.active_range_mut(*document, cancel)?;
                    *active = DomActiveRangeState::Known(None);
                    active.commit(cancel)?;
                }
                Err(DomError::StaleNode) => {
                    self.update_active_range_dependency(
                        *document,
                        DomActiveRangeState::Known(Some(id)),
                        DomActiveRangeState::Unavailable,
                        cancel,
                    )?;
                }
                Ok(_) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    pub(super) fn range_pre_remove(
        &mut self,
        node: Key,
        parent: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.counts.live_ranges == 0 {
            return Ok(());
        }
        let index = self.range_child_index(parent, node, cancel)?;
        let mut participants = self.range_participants(parent, cancel)?;
        let mut pending = custody::Held::new(
            Vec::new(),
            self.custody_ledger()
                .retain(custody::Usage::ZERO, |usage| {
                    self.admit_custody_usage(usage, cancel)
                })?,
        );
        self.range_temporary_push(&mut pending, node, cancel)?;
        while let Some(container) = pending.value_mut().pop() {
            self.source_step(cancel)?;
            self.add_range_participants(container, &mut participants, cancel)?;
            for child in self.node(container)?.children().iter() {
                self.range_temporary_push(&mut pending, child, cancel)?;
            }
        }
        for id in participants.iter() {
            self.source_step(cancel)?;
            let mut record = self.range_record(id)?;
            let old = record;
            for point in [&mut record.start, &mut record.end] {
                if self.range_inclusive_ancestor(node, point.container, cancel)? {
                    *point = Boundary {
                        container: parent,
                        offset: index,
                    };
                } else if point.container == parent && point.offset > index {
                    point.offset -= 1;
                }
            }
            if record != old {
                self.replace_range_record(id, record, cancel)?;
                #[cfg(test)]
                maintenance_observation::wrote();
            }
        }
        Ok(())
    }
    pub(super) fn range_insert(
        &mut self,
        parent: Key,
        before: Option<Key>,
        count: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(before) = before else {
            return Ok(());
        };
        if self.counts.live_ranges == 0 {
            return Ok(());
        }
        let index = self.range_child_index(parent, before, cancel)?;
        let participants = self.range_participants(parent, cancel)?;
        for id in participants.iter() {
            self.source_step(cancel)?;
            let mut record = self.range_record(id)?;
            let old = record;
            for point in [&mut record.start, &mut record.end] {
                if point.container == parent && point.offset > index {
                    point.offset = point
                        .offset
                        .checked_add(count)
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
            if record != old {
                self.replace_range_record(id, record, cancel)?;
                #[cfg(test)]
                maintenance_observation::wrote();
            }
        }
        Ok(())
    }
    pub(super) fn range_replace_data(
        &mut self,
        node: Key,
        offset: usize,
        removed: usize,
        inserted: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let end = offset
            .checked_add(removed)
            .ok_or(DomError::RepresentationExhausted)?;
        let participants = self.range_participants(node, cancel)?;
        for id in participants.iter() {
            self.source_step(cancel)?;
            let mut record = self.range_record(id)?;
            let old = record;
            for point in [&mut record.start, &mut record.end] {
                if point.container != node || point.offset <= offset {
                    continue;
                }
                point.offset = if point.offset <= end {
                    offset
                } else {
                    point
                        .offset
                        .checked_sub(removed)
                        .and_then(|n| n.checked_add(inserted))
                        .ok_or(DomError::RepresentationExhausted)?
                };
            }
            if record != old {
                self.replace_range_record(id, record, cancel)?;
                #[cfg(test)]
                maintenance_observation::wrote();
            }
        }
        Ok(())
    }
    pub(super) fn retire_range_container(
        &mut self,
        node: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let participants = self.range_participants(node, cancel)?;
        for id in participants.iter() {
            self.source_step(cancel)?;
            self.retire_range(id, cancel)?;
        }
        Ok(())
    }
    pub(super) fn has_live_ranges(&self, cancel: &AtomicBool) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        Ok(self.counts.live_ranges != 0)
    }
    /// Shared by genuine Source exposure preflight and all final publication fronts.
    pub(super) fn range_publication_ready(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        #[cfg(test)]
        if scale_metrics::ENABLED {
            scale_metrics::record(scale_metrics::Event::Entered(
                scale_metrics::Domain::Validation,
            ));
        }
        for (_, slot) in self.ranges.changed_records() {
            #[cfg(test)]
            if scale_metrics::ENABLED {
                scale_metrics::record(scale_metrics::Event::ValidationRecordVisited);
            }
            self.source_step(cancel)?;
            let Some(record) = slot
                .record
                .map(|record| record.map(|key| self.live_key(key)))
            else {
                continue;
            };
            if record.start.offset > self.range_node_length(record.start.container)?
                || record.end.offset > self.range_node_length(record.end.container)?
                || !matches!(
                    self.range_position(record.start, record.end, cancel)?,
                    Some(Position::Less | Position::Equal)
                )
            {
                return Err(DomError::RangePublicationUnavailable);
            }
        }
        Ok(())
    }

    /// The previous complete publication validator remains only as an oracle.
    /// It intentionally ignores touched metadata and reverse-index summaries.
    #[cfg(test)]
    fn full_range_publication_ready(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        for index in 0..self.ranges.len() {
            self.source_step(cancel)?;
            let Some(record) = self
                .ranges
                .slot(index)
                .and_then(|slot| slot.record)
                .map(|record| record.map(|key| self.live_key(key)))
            else {
                continue;
            };
            if record.start.offset > self.range_node_length(record.start.container)?
                || record.end.offset > self.range_node_length(record.end.container)?
                || !matches!(
                    self.raw_range_position(record.start, record.end, cancel)?,
                    Some(Position::Less | Position::Equal)
                )
            {
                return Err(DomError::RangePublicationUnavailable);
            }
        }
        for (index, _) in self.base.slots.iter() {
            self.source_step(cancel)?;
            let Ok(key) = self.existing_key(index) else {
                continue;
            };
            let Ok(node) = self.node(key) else { continue };
            if let DomActiveRangeState::Known(Some(id)) = node.active_range() {
                self.range_record(id)?;
            }
        }
        for index in 0..self.created.len() {
            self.source_step(cancel)?;
            let Ok(node) = self.node(Key::Created(index)) else {
                continue;
            };
            if let DomActiveRangeState::Known(Some(id)) = node.active_range() {
                self.range_record(id)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (Dom, DocumentId, DomRangeId) {
        let limits = DomLimits::new(
            16,
            1024,
            32,
            0,
            0,
            16,
            8192,
            crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_ranges(1)
        .unwrap();
        let mut dom = Dom::new(limits).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let range = batch.create_range(&document);
        batch.set_active_range(&document, Some((&range).into()));
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = commit.document(&document).unwrap();
        let range = commit.created_range(&range).unwrap().unwrap();
        (dom, document, range)
    }

    #[test]
    fn final_publication_cancellation_preserves_exact_range_and_association_then_retries() {
        let (mut dom, document, range) = fixture();
        let cancel = AtomicBool::new(false);
        let original = dom.state.clone();
        let stamp = dom.snapshot().stamp();
        let usage = dom.usage();
        let retire = || {
            let mut batch = DomBatch::new();
            batch.retire_range(range);
            batch
        };
        let prepared = dom.prepare(retire(), &cancel).unwrap();
        assert!(prepared.state.ranges[range.slot].record.is_none());
        assert_eq!(
            prepared
                .state
                .node(dom.owner, document.0)
                .unwrap()
                .active_range(),
            DomActiveRangeState::Known(None)
        );
        assert!(prepared.changes.contains(&DomChange::RangeRetired(range)));
        assert!(
            prepared
                .changes
                .contains(&DomChange::DocumentActiveRange(document))
        );
        cancel.store(true, Ordering::Relaxed);
        assert!(matches!(
            dom.publish(prepared, &cancel),
            Err(DomError::Cancelled)
        ));
        assert!(Arc::ptr_eq(&dom.state, &original));
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(dom.usage(), usage);
        cancel.store(false, Ordering::Relaxed);
        dom.apply(retire(), &cancel).unwrap();
        assert_eq!(
            dom.snapshot().document(document).unwrap().active_range(),
            DomActiveRangeState::Known(None)
        );
        let mut batch = DomBatch::new();
        let replacement = batch.create_range(document);
        let replacement = dom
            .apply(batch, &cancel)
            .unwrap()
            .created_range(&replacement)
            .unwrap()
            .unwrap();
        assert_ne!(replacement, range);
        assert_eq!(replacement.slot, range.slot);
        assert!(matches!(
            dom.snapshot().range(range),
            Err(DomError::StaleRange)
        ));
    }

    #[test]
    fn exhausted_range_generation_never_wraps_or_reuses_the_slot() {
        let (dom, document, range) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        // The owning counter boundary represents the state after repeated real
        // retire/reuse cycles; it supplies no tree or semantic observations.
        draft
            .set_active_range(Key::Existing(document.0), None, &cancel)
            .unwrap();
        let mut slot = draft.ranges.slot(range.slot).unwrap();
        slot.generation = u64::MAX - 1;
        draft.write_range(range.slot, slot, &cancel).unwrap();
        draft
            .retire_range(
                DomRangeId {
                    generation: u64::MAX - 1,
                    ..range
                },
                &cancel,
            )
            .unwrap();
        draft
            .create_range(Key::Existing(document.0), 0, dom.limits, &cancel)
            .unwrap();
        let last = draft.range_tickets[0];
        assert_eq!(last.generation, u64::MAX);
        draft.retire_range(last, &cancel).unwrap();
        assert_eq!(
            draft.create_range(Key::Existing(document.0), 1, dom.limits, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::Ranges,
                limit: 1
            })
        );
        assert!(draft.ranges.slot(range.slot).unwrap().retired);
        assert_eq!(draft.ranges.slot(range.slot).unwrap().generation, u64::MAX);
        assert_eq!(draft.range_record(last), Err(DomError::StaleRange));
    }

    #[test]
    fn private_retirement_and_reuse_do_not_alias_creation_tickets() {
        let (mut dom, document, range) = fixture();
        let cancel = AtomicBool::new(false);
        let mut retire = DomBatch::new();
        retire.retire_range(range);
        dom.apply(retire, &cancel).unwrap();
        let stamp = dom.snapshot().stamp();
        let mut batch = DomBatch::new();
        let first = batch.create_range(document);
        batch.retire_range(&first);
        let second = batch.create_range(document);
        batch.set_range_start(&first, document, 0);
        assert!(matches!(
            dom.apply(batch, &cancel),
            Err(DomError::StaleRange)
        ));
        assert_eq!(dom.snapshot().stamp(), stamp);
        let mut batch = DomBatch::new();
        let first = batch.create_range(document);
        batch.retire_range(&first);
        let second_live = batch.create_range(document);
        let commit = dom.apply(batch, &cancel).unwrap();
        assert_eq!(commit.created_range(&first).unwrap(), None);
        assert!(commit.created_range(&second_live).unwrap().is_some());
        assert_eq!(
            commit.created_range(&second).unwrap_err(),
            DomError::ForeignBatch
        );
    }
}
