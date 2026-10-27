use super::super::bank::{
    BankCustody, ControlledRecord, CustodyRegistry, Dependency, RecordDescription, Totals,
};
use super::super::custody::Usage;
use super::super::transaction::exact_forest;
use super::membership::{MembershipCustody, OwnedMembership};
use super::*;

// Semantic roots precede their matching owner-only custody on every drop path.
struct Pair<T> {
    value: Bank<T>,
    custody: BankCustody<T>,
}
impl<T> Clone for Pair<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            custody: self.custody.clone(),
        }
    }
}
impl<T> Default for Pair<T> {
    fn default() -> Self {
        Self {
            value: Bank::default(),
            custody: BankCustody::default(),
        }
    }
}
struct EntryWrite {
    shallow: Usage,
    summary: Totals,
    dependency: Option<Dependency>,
}
impl EntryWrite {
    fn plain(usage: Usage) -> Self {
        Self {
            shallow: usage,
            summary: Totals {
                bytes: usage.bytes(),
                units: usage.units(),
                occurrences: 1,
            },
            dependency: None,
        }
    }
    fn nested(shallow: Usage, summary: Usage, dependency: Dependency) -> Self {
        Self {
            shallow,
            summary: Totals {
                bytes: summary.bytes(),
                units: summary.units(),
                occurrences: 1,
            },
            dependency: Some(dependency),
        }
    }
}
impl<T> Pair<T> {
    fn set(
        &mut self,
        index: usize,
        usage: Usage,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        make: impl FnOnce() -> Result<T, DomError>,
    ) -> Result<(), DomError> {
        self.write(index, EntryWrite::plain(usage), registry, admit, make)
    }
    fn write(
        &mut self,
        index: usize,
        description: EntryWrite,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        make: impl FnOnce() -> Result<T, DomError>,
    ) -> Result<(), DomError> {
        let reserved = bank::Record::<T>::reserve()?;
        let debit = registry
            .ledger()
            .retain_shared(description.shallow, &mut *admit)?;
        let record = ControlledRecord::from_parts(reserved.materialize(make()?), debit);
        let record = match description.dependency {
            Some(value) => record.with_dependency(value),
            None => record,
        };
        let (value, custody) = self.value.set_controlled(
            &self.custody,
            index,
            record,
            false,
            description.summary,
            registry,
            admit,
        )?;
        self.value = value;
        self.custody = custody;
        Ok(())
    }
    fn remove(
        &mut self,
        index: usize,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let (value, custody, removed) =
            self.value
                .remove_controlled(&self.custody, index, registry, admit)?;
        self.value = value;
        self.custody = custody;
        drop(removed);
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum TransitionKey {
    Existing(DomViewTransitionId),
    Created(usize),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::store) enum RecordKey {
    Existing(DomViewTransitionRecordId),
    Created(usize),
}

#[derive(Clone)]
pub(in crate::store) struct DraftDocument {
    pub value: DomDocumentViewTransitions,
    pub active: Option<TransitionKey>,
}
#[derive(Clone)]
pub(in crate::store) struct DraftRecord {
    pub key: RecordKey,
    pub name: Arc<surgeist_css::values::view_transition::CssViewTransitionIdent>,
    pub branches: DomViewTransitionBranches,
    pub rules: Option<DomViewTransitionRules>,
    pub captured: [bool; 4],
}
pub(in crate::store) struct RecordRead<'a> {
    pub key: RecordKey,
    pub name: &'a Arc<surgeist_css::values::view_transition::CssViewTransitionIdent>,
    pub branches: DomViewTransitionBranches,
    pub rules: &'a Option<DomViewTransitionRules>,
    pub captured: [bool; 4],
}
impl RecordRead<'_> {
    pub fn owned(&self) -> DraftRecord {
        DraftRecord {
            key: self.key,
            name: self.name.clone(),
            branches: self.branches,
            rules: self.rules.clone(),
            captured: self.captured,
        }
    }
}
impl<'a> From<&'a DraftRecord> for RecordRead<'a> {
    fn from(r: &'a DraftRecord) -> Self {
        Self {
            key: r.key,
            name: &r.name,
            branches: r.branches,
            rules: &r.rules,
            captured: r.captured,
        }
    }
}
impl<'a> From<&'a Record> for RecordRead<'a> {
    fn from(r: &'a Record) -> Self {
        Self {
            key: RecordKey::Existing(r.id),
            name: &r.name,
            branches: r.branches,
            rules: &r.rules,
            captured: r.captured,
        }
    }
}
#[derive(Clone)]
pub(in crate::store) struct DraftTransition {
    pub key: TransitionKey,
    pub document: DocumentId,
    base: Pair<Record>,
    membership: OwnedMembership,
    changed: Pair<DraftRecord>,
    created: Pair<DraftRecord>,
    pub setup: bool,
    pub root_captured: bool,
    pub next_record: u64,
    pub name_units: usize,
    pub pseudos: usize,
}
impl DraftTransition {
    pub fn empty(key: TransitionKey, document: DocumentId) -> Self {
        Self {
            key,
            document,
            base: Pair::default(),
            membership: OwnedMembership::default(),
            changed: Pair::default(),
            created: Pair::default(),
            setup: false,
            root_captured: false,
            next_record: 0,
            name_units: 0,
            pseudos: 1,
        }
    }
    pub fn records(&self) -> impl Iterator<Item = RecordRead<'_>> {
        self.base
            .value
            .iter()
            .map(|(i, record)| {
                self.changed
                    .value
                    .get(i)
                    .map(RecordRead::from)
                    .unwrap_or_else(|| RecordRead::from(record))
            })
            .chain(self.created.value.values().map(RecordRead::from))
    }
    pub fn record(&self, key: RecordKey) -> Result<RecordRead<'_>, DomError> {
        let value = match key {
            RecordKey::Existing(id) => {
                let i = generation(id.generation)?;
                self.changed.value.get(i).map(RecordRead::from).or_else(|| {
                    self.base
                        .value
                        .get(i)
                        .filter(|r| r.id == id)
                        .map(RecordRead::from)
                })
            }
            RecordKey::Created(ticket) => self.created.value.get(ticket).map(RecordRead::from),
        };
        value.ok_or(match key {
            RecordKey::Existing(_) => DomError::StaleViewTransitionRecord,
            RecordKey::Created(_) => DomError::UnavailableCreation,
        })
    }
    pub fn record_count(&self) -> usize {
        self.base.value.record_count() + self.created.value.record_count()
    }
    pub fn name_owner(
        &self,
        name: &surgeist_css::values::view_transition::CssViewTransitionIdent,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<RecordKey>, DomError> {
        self.membership
            .value
            .name_owner(name, |key| Ok(self.record(key)?.name.as_ref()), step)
    }
    pub fn element_owner(
        &self,
        node: NodeId,
        step: &mut dyn FnMut() -> Result<(), DomError>,
    ) -> Result<Option<RecordKey>, DomError> {
        self.membership.value.element_owner(
            node,
            |key| {
                self.record(key)?
                    .branches
                    .new_element()
                    .ok_or(DomError::RepresentationExhausted)
            },
            step,
        )
    }
    pub fn change_record(
        &mut self,
        key: RecordKey,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
        change: impl FnOnce(RecordRead<'_>) -> DraftRecord,
    ) -> Result<(), DomError> {
        let read = self.record(key)?;
        let old_element = read.branches.new_element();
        let usage = record_usage::<DraftRecord>(read.name)?;
        let mut next = self.clone();
        let (table, index) = match key {
            RecordKey::Existing(id) => (&mut next.changed, generation(id.generation)?),
            RecordKey::Created(ticket) => (&mut next.created, ticket),
        };
        table.set(index, usage, registry, admit, || Ok(change(read)))?;
        let new_element = next.record(key)?.branches.new_element();
        if old_element != new_element {
            let mut membership = next.membership.clone();
            if let Some(node) = old_element {
                membership.replace_element(
                    node,
                    Some(key),
                    None,
                    |key| {
                        self.record(key)?
                            .branches
                            .new_element()
                            .ok_or(DomError::RepresentationExhausted)
                    },
                    exact_forest::Edit {
                        registry,
                        admit: &mut *admit,
                        step: &mut *step,
                    },
                )?;
            }
            if let Some(node) = new_element {
                membership.replace_element(
                    node,
                    None,
                    Some(key),
                    |key| {
                        next.record(key)?
                            .branches
                            .new_element()
                            .ok_or(DomError::RepresentationExhausted)
                    },
                    exact_forest::Edit {
                        registry,
                        admit: &mut *admit,
                        step: &mut *step,
                    },
                )?;
            }
            next.membership = membership;
        }
        *self = next;
        Ok(())
    }
    pub fn append_record(
        &mut self,
        ticket: usize,
        name: &Arc<surgeist_css::values::view_transition::CssViewTransitionIdent>,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        step: &mut dyn FnMut() -> Result<(), DomError>,
        observe: impl FnOnce() -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let incoming = record_usage::<()>(name)?;
        let reserved = bank::Record::<DraftRecord>::reserve()?;
        let debit = registry.ledger().retain(incoming, &mut *admit)?;
        let mut held = custody::Held::new(name.clone(), debit);
        observe()?;
        held.debit_mut()
            .grow(inline::<DraftRecord>(), &mut *admit)?;
        let shared = held.debit_mut().share(&mut *admit)?;
        let (owned_name, empty) = held.into_parts();
        drop(empty);
        let record = ControlledRecord::from_parts(
            reserved.materialize(DraftRecord {
                key: RecordKey::Created(ticket),
                name: owned_name,
                branches: DomViewTransitionBranches::Pending,
                rules: None,
                captured: [false; 4],
            }),
            shared,
        );
        let usage = record_usage::<DraftRecord>(record.value().name.as_ref())?;
        let mut next = self.clone();
        let (value, custody) = next.created.value.set_controlled(
            &next.created.custody,
            ticket,
            record,
            false,
            Totals {
                bytes: usage.bytes(),
                units: usage.units(),
                occurrences: 1,
            },
            registry,
            &mut *admit,
        )?;
        next.created.value = value;
        next.created.custody = custody;
        let mut membership = next.membership.clone();
        membership.replace_name(
            name,
            None,
            RecordKey::Created(ticket),
            |key| Ok(next.record(key)?.name.as_ref()),
            exact_forest::Edit {
                registry,
                admit: &mut *admit,
                step,
            },
        )?;
        next.membership = membership;
        *self = next;
        Ok(())
    }
}

#[derive(Clone, Default)]
pub(in crate::store) struct TransitionCustody {
    pub(in crate::store) documents: BankCustody<DomDocumentViewTransitions>,
    pub(in crate::store) transitions: BankCustody<Transition>,
    pub(in crate::store) document_transitions: BankCustody<DocumentTransitions>,
}

#[derive(Clone)]
pub(in crate::store) struct DraftTransitions {
    base: ViewTransitions,
    base_custody: TransitionCustody,
    documents: Pair<DraftDocument>,
    document_order: Pair<DocumentId>,
    pub next_document: usize,
    changed: Pair<Option<DraftTransition>>,
    created: Pair<DraftTransition>,
    record_owners: Pair<TransitionKey>,
    published_transitions: Pair<DomViewTransitionId>,
    published_records: Pair<DomViewTransitionRecordId>,
    pub transition_tickets: usize,
    pub record_tickets: usize,
    pub counts: (usize, usize, usize),
}
impl DraftTransitions {
    pub(in crate::store) fn capture(value: &ViewTransitions, custody: &TransitionCustody) -> Self {
        Self {
            base: value.clone(),
            base_custody: custody.clone(),
            documents: Pair::default(),
            document_order: Pair::default(),
            next_document: value.next_document,
            changed: Pair::default(),
            created: Pair::default(),
            record_owners: Pair::default(),
            published_transitions: Pair::default(),
            published_records: Pair::default(),
            transition_tickets: 0,
            record_tickets: 0,
            counts: value.usage(),
        }
    }
    #[cfg(test)]
    pub fn usage(&self) -> Result<(usize, usize, usize), DomError> {
        Ok(self.counts)
    }
    pub fn document(&self, document: DocumentId) -> Result<DraftDocument, DomError> {
        if let Some(value) = self
            .documents
            .value
            .get(document.node().slot)
            .filter(|d| d.value.document == document)
        {
            return Ok(value.clone());
        }
        let value = self
            .base
            .documents
            .get(document.node().slot)
            .filter(|d| d.document == document)
            .ok_or(DomError::ViewTransitionsUninitialized)?;
        Ok(DraftDocument {
            value: value.clone(),
            active: value.active.map(TransitionKey::Existing),
        })
    }
    pub fn set_document(
        &mut self,
        document: DraftDocument,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let id = document.value.document;
        let ordinal = document.value.init_ordinal;
        if self.document_order.value.get(ordinal) != Some(&id) {
            self.document_order
                .set(ordinal, inline::<DocumentId>(), registry, admit, || Ok(id))?;
        }
        self.documents.set(
            document.value.document.node().slot,
            inline::<DraftDocument>(),
            registry,
            admit,
            || Ok(document),
        )
    }
    pub fn get(&self, key: TransitionKey) -> Result<DraftTransition, DomError> {
        match key {
            TransitionKey::Created(ticket) => self
                .created
                .value
                .get(ticket)
                .cloned()
                .ok_or(DomError::UnavailableCreation),
            TransitionKey::Existing(id) => {
                let index = generation(id.generation)?;
                if let Some(value) = self.changed.value.get(index) {
                    return value.clone().ok_or(DomError::StaleViewTransition);
                }
                let value = self
                    .base
                    .transitions
                    .get(index)
                    .filter(|t| t.id == id)
                    .ok_or(DomError::StaleViewTransition)?;
                let record = self
                    .base
                    .transitions
                    .controlled_record(&self.base_custody.transitions, index)
                    .expect("canonical transition has custody");
                let records = record
                    .dependency::<TransitionChildren>()
                    .expect("transition custody retains its participant roots");
                Ok(DraftTransition {
                    key,
                    document: value.document,
                    base: records.records.clone(),
                    membership: OwnedMembership {
                        value: value.membership.clone(),
                        custody: records.membership.clone(),
                    },
                    changed: Pair::default(),
                    created: Pair::default(),
                    setup: value.setup,
                    root_captured: value.root_captured,
                    next_record: value.next_record,
                    name_units: value.name_units,
                    pseudos: value.pseudos,
                })
            }
        }
    }
    pub fn set(
        &mut self,
        value: DraftTransition,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        match value.key {
            TransitionKey::Existing(id) => self.changed.set(
                generation(id.generation)?,
                inline::<Option<DraftTransition>>(),
                registry,
                admit,
                || Ok(Some(value)),
            ),
            TransitionKey::Created(ticket) => {
                self.created
                    .set(ticket, inline::<DraftTransition>(), registry, admit, || {
                        Ok(value)
                    })
            }
        }
    }
    pub fn remove(
        &mut self,
        key: TransitionKey,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        match key {
            TransitionKey::Existing(id) => self.changed.set(
                generation(id.generation)?,
                inline::<Option<DraftTransition>>(),
                registry,
                admit,
                || Ok(None),
            ),
            TransitionKey::Created(ticket) => self.created.remove(ticket, registry, admit),
        }
    }
    pub fn transition(
        &self,
        owner: &Arc<BatchIdentity>,
        store: DomOwnerId,
        operand: ViewTransitionOperand,
    ) -> Result<TransitionKey, DomError> {
        let created = matches!(&operand, ViewTransitionOperand::Created(_));
        let key = match operand {
            ViewTransitionOperand::Existing(id) => {
                if id.owner != store {
                    return Err(DomError::ForeignOwner);
                }
                TransitionKey::Existing(id)
            }
            ViewTransitionOperand::Created(ticket) => {
                if !Arc::ptr_eq(owner, &ticket.owner) {
                    return Err(DomError::ForeignBatch);
                }
                self.published_transitions
                    .value
                    .get(ticket.index)
                    .copied()
                    .map(TransitionKey::Existing)
                    .unwrap_or(TransitionKey::Created(ticket.index))
            }
        };
        self.get(key).map_err(|error| {
            if created && matches!(error, DomError::StaleViewTransition) {
                DomError::UnavailableCreation
            } else {
                error
            }
        })?;
        Ok(key)
    }
    pub fn record(
        &self,
        owner: &Arc<BatchIdentity>,
        store: DomOwnerId,
        operand: ViewTransitionRecordOperand,
    ) -> Result<(TransitionKey, RecordKey), DomError> {
        let created = matches!(&operand, ViewTransitionRecordOperand::Created(_));
        let (transition, record) = match operand {
            ViewTransitionRecordOperand::Existing(id) => {
                if id.transition.owner != store {
                    return Err(DomError::ForeignOwner);
                }
                (
                    TransitionKey::Existing(id.transition),
                    RecordKey::Existing(id),
                )
            }
            ViewTransitionRecordOperand::Created(ticket) => {
                if !Arc::ptr_eq(owner, &ticket.owner) {
                    return Err(DomError::ForeignBatch);
                }
                if let Some(id) = self.published_records.value.get(ticket.index) {
                    (
                        TransitionKey::Existing(id.transition),
                        RecordKey::Existing(*id),
                    )
                } else {
                    (
                        *self
                            .record_owners
                            .value
                            .get(ticket.index)
                            .ok_or(DomError::UnavailableCreation)?,
                        RecordKey::Created(ticket.index),
                    )
                }
            }
        };
        self.get(transition)
            .and_then(|value| value.record(record).map(|_| ()))
            .map_err(|error| match error {
                DomError::StaleViewTransition
                | DomError::StaleViewTransitionRecord
                | DomError::UnavailableCreation => {
                    if created {
                        DomError::UnavailableCreation
                    } else {
                        DomError::StaleViewTransitionRecord
                    }
                }
                error => error,
            })?;
        Ok((transition, record))
    }
    pub fn record_owner(
        &mut self,
        ticket: usize,
        key: TransitionKey,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        self.record_owners
            .set(ticket, inline::<TransitionKey>(), registry, admit, || {
                Ok(key)
            })
    }
    pub fn checked_document(
        &self,
        key: TransitionKey,
        document: DocumentId,
    ) -> Result<(), DomError> {
        if self.get(key)?.document == document {
            Ok(())
        } else {
            Err(DomError::WrongDocument)
        }
    }
    pub fn mutable_topology(
        &self,
        key: TransitionKey,
        document: DocumentId,
    ) -> Result<(), DomError> {
        self.checked_document(key, document)?;
        if self.get(key)?.setup || self.document(document)?.active != Some(key) {
            Err(DomError::InvalidViewTransitionState)
        } else {
            Ok(())
        }
    }
}

pub(in crate::store) fn generation(value: u64) -> Result<usize, DomError> {
    usize::try_from(value).map_err(|_| DomError::RepresentationExhausted)
}
fn inline<T>() -> Usage {
    Usage::new(size_of::<T>(), 0)
}
fn record_usage<T>(
    name: &surgeist_css::values::view_transition::CssViewTransitionIdent,
) -> Result<Usage, DomError> {
    let bytes = size_of::<T>()
        .checked_add(size_of::<
            surgeist_css::values::view_transition::CssViewTransitionIdent,
        >())
        .and_then(|n| n.checked_add(name.retained_buffer_bytes()))
        .ok_or(DomError::RepresentationExhausted)?;
    Ok(Usage::new(bytes, 0))
}

pub(in crate::store) struct TransitionUndo {
    previous: DraftTransitions,
    units: usize,
}
impl TransitionUndo {
    /// Captured paired roots alias existing controlled objects. Their inline
    /// handles are charged by the enclosing journal entry.
    #[cfg(test)]
    pub fn journal_bytes(&self) -> usize {
        0
    }
    pub fn capture(draft: &Draft) -> Self {
        Self {
            previous: draft.view_transitions.clone(),
            units: draft.units,
        }
    }
    pub fn restore(self, draft: &mut Draft) {
        draft.view_transitions = self.previous;
        draft.units = self.units;
        let (transitions, records, pseudos) = draft.view_transitions.counts;
        draft.counts.transitions = transitions;
        draft.counts.records = records;
        draft.counts.pseudos = pseudos;
    }
}
#[derive(Clone)]
pub(in crate::store) struct TransitionAck {
    value: DraftTransitions,
}
impl TransitionAck {
    pub fn apply(self, target: &mut DraftTransitions) {
        *target = self.value;
    }
}
pub(in crate::store) struct FinishedTransitions {
    pub value: ViewTransitions,
    pub custody: TransitionCustody,
    pub transition_ids: Vec<Option<DomViewTransitionId>>,
    pub record_ids: Vec<Option<DomViewTransitionRecordId>>,
    pub changes: custody::Held<Vec<DomChange>>,
    pub ack: TransitionAck,
    pub output_custody: custody::Debit,
}

fn reserve_output<T>(
    value: &mut Vec<T>,
    length: usize,
    debit: &mut custody::Debit,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let bytes = length
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    debit.grow(Usage::new(bytes, 0), &mut *admit)?;
    #[cfg(test)]
    let previous_capacity = value.capacity();
    value
        .try_reserve_exact(length)
        .map_err(|_| DomError::AllocationFailed)?;
    let actual = value
        .capacity()
        .checked_mul(size_of::<T>())
        .ok_or(DomError::RepresentationExhausted)?;
    #[cfg(test)]
    if super::super::scale_metrics::ENABLED && value.capacity() != previous_capacity && actual != 0
    {
        super::super::scale_metrics::record(
            super::super::scale_metrics::Event::StructuralAllocation { bytes: actual },
        );
    }
    debit.grow(Usage::new(actual - bytes, 0), admit)?;
    Ok(())
}

impl DraftTransitions {
    pub(in crate::store) fn finish(
        &self,
        owner: DomOwnerId,
        original: &ViewTransitions,
        original_custody: &TransitionCustody,
        registry: &CustodyRegistry,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<FinishedTransitions, DomError> {
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        let mut outputs = registry.ledger().retain(Usage::ZERO, &mut admit)?;
        let mut transition_ids = Vec::new();
        let mut record_ids = Vec::new();
        let mut changes_charge = registry.ledger().retain(Usage::ZERO, &mut admit)?;
        let mut changes = Vec::new();
        reserve_output(
            &mut transition_ids,
            self.transition_tickets,
            &mut outputs,
            &mut admit,
        )?;
        reserve_output(
            &mut record_ids,
            self.record_tickets,
            &mut outputs,
            &mut admit,
        )?;
        transition_ids.resize(self.transition_tickets, None);
        #[cfg(test)]
        written.extended::<Option<DomViewTransitionId>>(self.transition_tickets);
        record_ids.resize(self.record_tickets, None);
        #[cfg(test)]
        written.extended::<Option<DomViewTransitionRecordId>>(self.record_tickets);
        for (ticket, id) in self.published_transitions.value.iter() {
            step()?;
            transition_ids[ticket] = Some(*id);
        }
        for (ticket, id) in self.published_records.value.iter() {
            step()?;
            record_ids[ticket] = Some(*id);
        }
        let mut bound = self.documents.value.record_count();
        for (_, value) in self.changed.value.iter() {
            step()?;
            bound = bound
                .checked_add(2)
                .ok_or(DomError::RepresentationExhausted)?;
            let records = value
                .as_ref()
                .map(|t| t.changed.value.record_count() + t.created.value.record_count())
                .unwrap_or(0);
            bound = bound
                .checked_add(
                    records
                        .checked_mul(5)
                        .ok_or(DomError::RepresentationExhausted)?,
                )
                .ok_or(DomError::RepresentationExhausted)?;
        }
        for (index, value) in self.changed.value.iter() {
            if value.is_none() {
                let count = original
                    .transitions
                    .get(index)
                    .map(|t| t.records.record_count())
                    .unwrap_or(0);
                bound = bound
                    .checked_add(count)
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        for (_, value) in self.created.value.iter() {
            step()?;
            bound = bound
                .checked_add(1 + value.record_count())
                .ok_or(DomError::RepresentationExhausted)?;
        }
        reserve_output(&mut changes, bound, &mut changes_charge, &mut admit)?;

        let mut value = original.clone();
        let mut custody = original_custody.clone();
        let mut transitions = Pair {
            value: original.transitions.clone(),
            custody: original_custody.transitions.clone(),
        };
        let mut published_transitions = self.published_transitions.clone();
        let mut published_records = self.published_records.clone();
        // Assign only surviving private tickets. A private begin/append/release
        // consumes no published generation, including across Source phases.
        for (ticket, transition) in self.created.value.iter() {
            step()?;
            let published_generation = value.next_transition;
            value.next_transition = published_generation
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            let id = DomViewTransitionId {
                owner,
                generation: published_generation,
            };
            generation(id.generation)?;
            transition_ids[ticket] = Some(id);
            published_transitions.set(
                ticket,
                inline::<DomViewTransitionId>(),
                registry,
                &mut admit,
                || Ok(id),
            )?;
            let _ = transition;
        }
        for (index, selected) in self.changed.value.iter() {
            step()?;
            let before = original
                .transitions
                .get(index)
                .expect("touched published transition");
            if let Some(selected) = selected {
                let SealedTransition {
                    value: after,
                    records: pair,
                    changes: changed_records,
                } = selected.seal(
                    before.id,
                    &mut record_ids,
                    &mut published_records,
                    registry,
                    &mut admit,
                    &mut step,
                )?;
                if transition_fields_equal(before, &after) && changed_records.value().is_empty() {
                    continue;
                }
                transition_changes(
                    Some(before),
                    Some(&after),
                    changed_records.value(),
                    &mut changes,
                    &mut step,
                )?;
                let summary = transition_usage(&after)?;
                transitions.write(
                    index,
                    EntryWrite::nested(
                        inline::<Transition>(),
                        summary,
                        Dependency::new(pair, registry.ledger(), &mut admit)?,
                    ),
                    registry,
                    &mut admit,
                    || Ok(after),
                )?;
            } else {
                transition_changes(Some(before), None, &[], &mut changes, &mut step)?;
                transitions.remove(index, registry, &mut admit)?;
                remove_document_member(
                    &mut value,
                    &mut custody,
                    before.document,
                    before.id,
                    registry,
                    &mut admit,
                )?;
            }
        }
        for (ticket, selected) in self.created.value.iter() {
            step()?;
            let id = transition_ids[ticket].expect("surviving transition assigned");
            let index = generation(id.generation)?;
            let SealedTransition {
                value: after,
                records: pair,
                changes: changed_records,
            } = selected.seal(
                id,
                &mut record_ids,
                &mut published_records,
                registry,
                &mut admit,
                &mut step,
            )?;
            transition_changes(
                None,
                Some(&after),
                changed_records.value(),
                &mut changes,
                &mut step,
            )?;
            let summary = transition_usage(&after)?;
            transitions.write(
                index,
                EntryWrite::nested(
                    inline::<Transition>(),
                    summary,
                    Dependency::new(pair, registry.ledger(), &mut admit)?,
                ),
                registry,
                &mut admit,
                || Ok(after),
            )?;
            add_document_member(
                &mut value,
                &mut custody,
                selected.document,
                id,
                registry,
                &mut admit,
            )?;
        }
        value.transitions = transitions.value;
        custody.transitions = transitions.custody;
        let mut documents = Pair {
            value: value.documents.clone(),
            custody: custody.documents.clone(),
        };
        // Document deltas precede transition deltas, preserving the owning
        // domain's established ordering without visiting unrelated documents.
        let transition_change_count = changes.len();
        for (_, document) in self.document_order.value.iter() {
            step()?;
            let index = document.node().slot;
            let selected = self
                .documents
                .value
                .get(index)
                .expect("selected Document header");
            let active = match selected.active {
                Some(TransitionKey::Existing(id)) => Some(id),
                Some(TransitionKey::Created(ticket)) => transition_ids[ticket],
                None => None,
            };
            if original.documents.get(index).is_some_and(|old| {
                old.document == selected.value.document
                    && old.sheet == selected.value.sheet
                    && old.suppressed == selected.value.suppressed
                    && old.show == selected.value.show
                    && old.active == active
            }) {
                continue;
            }
            documents.set(
                index,
                inline::<DomDocumentViewTransitions>(),
                registry,
                &mut admit,
                || {
                    let mut result = selected.value.clone();
                    result.active = active;
                    Ok(result)
                },
            )?;
            changes.push(DomChange::ViewTransitionDocument(selected.value.document));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
        changes.rotate_left(transition_change_count);
        #[cfg(test)]
        if transition_change_count != 0 && transition_change_count < changes.len() {
            // Count the completed logical sequence reorder once, excluding
            // std's internal swaps/cycles and fixed ticket-cell overwrites.
            written.extended::<DomChange>(changes.len());
        }
        value.documents = documents.value;
        custody.documents = documents.custody;
        value.next_document = self.next_document;
        value.records = self.counts.1;
        value.pseudos = self.counts.2;
        value.recount_retained()?;
        let mut acknowledged = Self::capture(&value, &custody);
        acknowledged.transition_tickets = self.transition_tickets;
        acknowledged.record_tickets = self.record_tickets;
        acknowledged.published_transitions = published_transitions;
        acknowledged.published_records = published_records;
        Ok(FinishedTransitions {
            value,
            custody,
            transition_ids,
            record_ids,
            changes: custody::Held::new(changes, changes_charge),
            ack: TransitionAck {
                value: acknowledged,
            },
            output_custody: outputs,
        })
    }
}
// The canonical semantic root precedes its matching owner-only record custody;
// the changed-ID buffer carries its own affine charge through consumption.
struct SealedTransition {
    value: Transition,
    records: TransitionChildren,
    changes: custody::Held<Vec<DomViewTransitionRecordId>>,
}
#[derive(Clone)]
struct TransitionChildren {
    records: Pair<Record>,
    membership: MembershipCustody,
}
impl DraftTransition {
    fn sealed_record<'a>(
        &'a self,
        records: &'a Bank<Record>,
        key: RecordKey,
    ) -> Result<RecordRead<'a>, DomError> {
        match key {
            RecordKey::Existing(id) => records
                .get(generation(id.generation)?)
                .filter(|record| record.id == id)
                .map(RecordRead::from)
                .ok_or(DomError::StaleViewTransitionRecord),
            RecordKey::Created(_) => self.record(key),
        }
    }
}
impl DraftTransition {
    fn seal(
        &self,
        id: DomViewTransitionId,
        ids: &mut [Option<DomViewTransitionRecordId>],
        published: &mut Pair<DomViewTransitionRecordId>,
        registry: &CustodyRegistry,
        admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
        step: &mut impl FnMut() -> Result<(), DomError>,
    ) -> Result<SealedTransition, DomError> {
        let mut pair = self.base.clone();
        let mut membership = self.membership.clone();
        let mut next_record = self.next_record;
        #[cfg(test)]
        let mut written = crate::store::CopyObservation::default();
        let mut changed = Vec::new();
        // This bounded scratch holds identities only. Its actual allocation is
        // prepaid and retained until the returned records have been consumed.
        let mut scratch = registry.ledger().retain(Usage::ZERO, &mut *admit)?;
        reserve_output(
            &mut changed,
            self.changed.value.record_count() + self.created.value.record_count(),
            &mut scratch,
            admit,
        )?;
        for (index, selected) in self.changed.value.iter() {
            step()?;
            let old = self.base.value.get(index).expect("existing participant");
            if record_fields_equal(old, selected) {
                continue;
            }
            let record_id = old.id;
            pair.set(
                index,
                record_usage::<Record>(&selected.name)?,
                registry,
                admit,
                || {
                    Ok(Record {
                        id: record_id,
                        name: selected.name.clone(),
                        branches: selected.branches,
                        rules: selected.rules.clone(),
                        captured: selected.captured,
                    })
                },
            )?;
            changed.push(record_id);
            #[cfg(test)]
            written.pushed::<DomViewTransitionRecordId>();
        }
        for (ticket, selected) in self.created.value.iter() {
            step()?;
            let record_id = DomViewTransitionRecordId {
                transition: id,
                generation: next_record,
            };
            next_record = next_record
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            let index = generation(record_id.generation)?;
            ids[ticket] = Some(record_id);
            published.set(
                ticket,
                inline::<DomViewTransitionRecordId>(),
                registry,
                admit,
                || Ok(record_id),
            )?;
            pair.set(
                index,
                record_usage::<Record>(&selected.name)?,
                registry,
                admit,
                || {
                    Ok(Record {
                        id: record_id,
                        name: selected.name.clone(),
                        branches: selected.branches,
                        rules: selected.rules.clone(),
                        captured: selected.captured,
                    })
                },
            )?;
            membership.replace_name(
                &selected.name,
                Some(RecordKey::Created(ticket)),
                RecordKey::Existing(record_id),
                |key| Ok(self.sealed_record(&pair.value, key)?.name.as_ref()),
                exact_forest::Edit {
                    registry,
                    admit: &mut *admit,
                    step: &mut *step,
                },
            )?;
            if let Some(node) = selected.branches.new_element() {
                membership.replace_element(
                    node,
                    Some(RecordKey::Created(ticket)),
                    Some(RecordKey::Existing(record_id)),
                    |key| {
                        self.sealed_record(&pair.value, key)?
                            .branches
                            .new_element()
                            .ok_or(DomError::RepresentationExhausted)
                    },
                    exact_forest::Edit {
                        registry,
                        admit: &mut *admit,
                        step: &mut *step,
                    },
                )?;
            }
            changed.push(record_id);
            #[cfg(test)]
            written.pushed::<DomViewTransitionRecordId>();
        }
        // Scratch is transferred with its vector to the caller below.
        // The caller consumes it synchronously before another admission.
        let value = Transition {
            id,
            document: self.document,
            records: pair.value.clone(),
            membership: membership.value,
            setup: self.setup,
            root_captured: self.root_captured,
            next_record,
            name_units: self.name_units,
            pseudos: self.pseudos,
        };
        Ok(SealedTransition {
            value,
            records: TransitionChildren {
                records: pair,
                membership: membership.custody,
            },
            changes: custody::Held::new(changed, scratch),
        })
    }
}
fn record_fields_equal(old: &Record, new: &DraftRecord) -> bool {
    Arc::ptr_eq(&old.name, &new.name)
        && old.branches == new.branches
        && old.rules == new.rules
        && old.captured == new.captured
}
fn transition_fields_equal(old: &Transition, new: &Transition) -> bool {
    old.id == new.id
        && old.document == new.document
        && old.setup == new.setup
        && old.root_captured == new.root_captured
        && old.next_record == new.next_record
}
fn transition_usage(value: &Transition) -> Result<Usage, DomError> {
    inline::<Transition>()
        .checked_add(bank_usage(&value.records)?)?
        .checked_add(value.membership.heap_usage()?)
}
fn bank_usage<T>(value: &Bank<T>) -> Result<Usage, DomError> {
    Usage::new(value.index_charge().bytes, 0)
        .checked_add(Usage::new(value.totals().bytes, value.totals().units))
}
impl ViewTransitions {
    /// A pruned controlled page retains its records' strong owner-only child
    /// dependencies. Only previously uncontrolled semantic frontiers enter
    /// these descriptors; active nested records never require a second walk.
    pub(in crate::store) fn adopt_custody(
        &self,
        registry: &CustodyRegistry,
        mut admit: impl FnMut(Usage) -> Result<(), DomError>,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<TransitionCustody, DomError> {
        let admission = std::cell::RefCell::new(&mut admit);
        let documents = self.documents.adopt_controlled(
            registry,
            |_| {
                step()?;
                Ok(inline::<DomDocumentViewTransitions>())
            },
            |usage| admission.borrow_mut()(usage),
        )?;
        let transitions = self.transitions.adopt_controlled(
            registry,
            |transition| {
                step()?;
                let custody = transition.records.adopt_controlled(
                    registry,
                    |record| {
                        step()?;
                        record_usage::<Record>(&record.name)
                    },
                    |usage| admission.borrow_mut()(usage),
                )?;
                let pair = Pair {
                    value: transition.records.clone(),
                    custody,
                };
                let membership = transition.membership.adopt_custody(
                    registry,
                    &mut |usage| admission.borrow_mut()(usage),
                    &mut step,
                )?;
                let dependency = Dependency::new(
                    TransitionChildren {
                        records: pair,
                        membership,
                    },
                    registry.ledger(),
                    |usage| admission.borrow_mut()(usage),
                )?;
                Ok(RecordDescription::with_dependency(
                    inline::<Transition>(),
                    dependency,
                ))
            },
            |usage| admission.borrow_mut()(usage),
        )?;
        let document_transitions = self.document_transitions.adopt_controlled(
            registry,
            |document| {
                step()?;
                let custody = document.ids.adopt_controlled(
                    registry,
                    |_| {
                        step()?;
                        Ok(inline::<DomViewTransitionId>())
                    },
                    |usage| admission.borrow_mut()(usage),
                )?;
                let pair = Pair {
                    value: document.ids.clone(),
                    custody,
                };
                let dependency = Dependency::new(pair, registry.ledger(), |usage| {
                    admission.borrow_mut()(usage)
                })?;
                Ok(RecordDescription::with_dependency(
                    inline::<DocumentTransitions>(),
                    dependency,
                ))
            },
            |usage| admission.borrow_mut()(usage),
        )?;
        Ok(TransitionCustody {
            documents,
            transitions,
            document_transitions,
        })
    }
    pub(in crate::store) fn counts(&self) -> (usize, usize, usize) {
        (self.transitions.record_count(), self.records, self.pseudos)
    }
    pub(in crate::store) fn usage(&self) -> (usize, usize, usize) {
        self.counts()
    }
    #[cfg(test)]
    pub(in crate::store) fn retained_usage(&self) -> Result<Usage, DomError> {
        Ok(Usage::new(self.retained_bytes, self.retained_units))
    }
    fn recount_retained(&mut self) -> Result<(), DomError> {
        let usage = bank_usage(&self.documents)?
            .checked_add(bank_usage(&self.transitions)?)?
            .checked_add(bank_usage(&self.document_transitions)?)?;
        self.retained_bytes = usage.bytes();
        self.retained_units = usage.units();
        Ok(())
    }
    pub(crate) fn document(&self, id: DocumentId) -> Option<&DomDocumentViewTransitions> {
        self.documents
            .get(id.node().slot)
            .filter(|d| d.document == id)
    }
    pub(in crate::store) fn origin_transitions(
        &self,
        document: DocumentId,
    ) -> impl Iterator<Item = DomViewTransitionId> + '_ {
        self.document_transitions
            .get(document.node().slot)
            .filter(|entry| entry.document == document)
            .into_iter()
            .flat_map(|entry| entry.ids.values().copied())
    }
}
fn add_document_member(
    value: &mut ViewTransitions,
    custody: &mut TransitionCustody,
    document: DocumentId,
    id: DomViewTransitionId,
    registry: &CustodyRegistry,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    change_document_member(value, custody, document, id, true, registry, admit)
}
fn remove_document_member(
    value: &mut ViewTransitions,
    custody: &mut TransitionCustody,
    document: DocumentId,
    id: DomViewTransitionId,
    registry: &CustodyRegistry,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    change_document_member(value, custody, document, id, false, registry, admit)
}
fn change_document_member(
    value: &mut ViewTransitions,
    custody: &mut TransitionCustody,
    document: DocumentId,
    id: DomViewTransitionId,
    add: bool,
    registry: &CustodyRegistry,
    admit: &mut impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<(), DomError> {
    let index = document.node().slot;
    let existing = value
        .document_transitions
        .get(index)
        .filter(|entry| entry.document == document);
    if !add && existing.is_none() {
        return Ok(());
    }
    let mut pair = if existing.is_some() {
        let record = value
            .document_transitions
            .controlled_record(&custody.document_transitions, index)
            .expect("document membership has custody");
        let retained = record
            .dependency::<Pair<DomViewTransitionId>>()
            .expect("document membership retains child roots");
        Pair::clone(&retained)
    } else {
        Pair::default()
    };
    let generation = generation(id.generation)?;
    if add {
        pair.set(
            generation,
            inline::<DomViewTransitionId>(),
            registry,
            admit,
            || Ok(id),
        )?;
    } else {
        pair.remove(generation, registry, admit)?;
    }
    let mut roots = Pair {
        value: value.document_transitions.clone(),
        custody: custody.document_transitions.clone(),
    };
    let ids = pair.value.clone();
    let summary = inline::<DocumentTransitions>().checked_add(bank_usage(&ids)?)?;
    roots.write(
        index,
        EntryWrite::nested(
            inline::<DocumentTransitions>(),
            summary,
            Dependency::new(pair, registry.ledger(), &mut *admit)?,
        ),
        registry,
        admit,
        || Ok(DocumentTransitions { document, ids }),
    )?;
    value.document_transitions = roots.value;
    custody.document_transitions = roots.custody;
    Ok(())
}

fn transition_changes(
    before: Option<&Transition>,
    after: Option<&Transition>,
    changed: &[DomViewTransitionRecordId],
    changes: &mut Vec<DomChange>,
    step: &mut impl FnMut() -> Result<(), DomError>,
) -> Result<(), DomError> {
    #[cfg(test)]
    let mut written = crate::store::CopyObservation::default();
    let id = before.or(after).expect("one transition").id;
    changes.push(DomChange::ViewTransition(id));
    #[cfg(test)]
    written.pushed::<DomChange>();
    if before.is_none() || after.is_none() {
        for record in &before.or(after).expect("one transition").records {
            step()?;
            changes.push(DomChange::ViewTransitionRecord(record.id));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
    } else {
        for id in changed {
            step()?;
            changes.push(DomChange::ViewTransitionRecord(*id));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
    }
    if let (Some(before), Some(after)) = (before, after) {
        if before.root_captured != after.root_captured {
            changes.push(DomChange::ViewTransitionCaptured(
                DomViewTransitionSubject::Pseudo(pseudo_id(
                    id,
                    None,
                    DomViewTransitionPseudoKind::Root,
                )),
            ));
            #[cfg(test)]
            written.pushed::<DomChange>();
        }
        for record_id in changed {
            let index = generation(record_id.generation)?;
            if let (Some(old), Some(new)) = (before.records.get(index), after.records.get(index)) {
                for kind in [
                    DomViewTransitionPseudoKind::Group,
                    DomViewTransitionPseudoKind::ImagePair,
                    DomViewTransitionPseudoKind::Old,
                    DomViewTransitionPseudoKind::New,
                ] {
                    if old.captured[flag_index(kind)] != new.captured[flag_index(kind)] {
                        changes.push(DomChange::ViewTransitionCaptured(
                            DomViewTransitionSubject::Pseudo(pseudo_id(id, Some(*record_id), kind)),
                        ));
                        #[cfg(test)]
                        written.pushed::<DomChange>();
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
pub(in crate::store) fn set_next_record_for_test(
    dom: &mut Dom,
    id: DomViewTransitionId,
    next: u64,
) {
    let index = generation(id.generation).unwrap();
    let mut pair = Pair {
        value: dom.state.view_transitions.transitions.clone(),
        custody: dom.state_custody.transitions.transitions.clone(),
    };
    let mut value = pair.value[index].clone();
    value.next_record = next;
    let summary = transition_usage(&value).unwrap();
    let previous = pair.value.controlled_record(&pair.custody, index).unwrap();
    let child = previous.dependency::<TransitionChildren>().unwrap();
    let dependency = Dependency::new(
        TransitionChildren::clone(&child),
        dom.custody_registry.ledger(),
        |_| Ok(()),
    )
    .unwrap();
    pair.write(
        index,
        EntryWrite::nested(inline::<Transition>(), summary, dependency),
        &dom.custody_registry,
        &mut |_| Ok(()),
        || Ok(value),
    )
    .unwrap();
    let mut frontiers = (*dom.state_custody).clone();
    frontiers.transitions.transitions = pair.custody;
    let state = Arc::get_mut(&mut dom.state).unwrap();
    state.identity = custody::MaterializationId::fresh().unwrap();
    state.view_transitions.transitions = pair.value;
    dom.state_custody =
        transaction::StateCustody::from_parts(&dom.state, frontiers, &dom.custody_registry, |_| {
            Ok(())
        })
        .unwrap();
}
