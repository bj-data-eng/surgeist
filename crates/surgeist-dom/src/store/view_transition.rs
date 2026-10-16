use super::*;
use crate::view_transition::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransitionKey {
    Existing(DomViewTransitionId),
    Created(usize),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RecordKey {
    Existing(DomViewTransitionRecordId),
    Created(usize),
}
struct DraftDocument {
    value: DomDocumentViewTransitions,
    active: Option<TransitionKey>,
}
struct DraftRecord {
    key: RecordKey,
    name: Arc<surgeist_css::CssViewTransitionIdent>,
    branches: DomViewTransitionBranches,
    rules: Option<DomViewTransitionRules>,
    captured: [bool; 4],
}
struct DraftTransition {
    key: TransitionKey,
    document: DocumentId,
    records: Vec<DraftRecord>,
    setup: bool,
    root_captured: bool,
    next_record: u64,
}
pub(super) struct DraftTransitions {
    documents: Vec<DraftDocument>,
    transitions: Vec<DraftTransition>,
    next_transition: u64,
    transition_tickets: usize,
    record_tickets: usize,
    published_transitions: Vec<Option<DomViewTransitionId>>,
    published_records: Vec<Option<DomViewTransitionRecordId>>,
}
impl DraftTransitions {
    pub(super) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        let mut payload = source::storage::Payload::default();
        for transition in &self.transitions {
            for record in &transition.records {
                step()?;
                payload.add_bytes(size_of::<surgeist_css::CssViewTransitionIdent>())?;
                payload.add_bytes(record.name.retained_buffer_bytes())?;
            }
        }
        Ok(payload)
    }
    pub(super) fn checkpoint(
        &self,
        a: &mut source::checkpoint::Admission<'_>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let mut documents = Vec::new();
        a.reserve(&mut documents, self.documents.len())?;
        for d in &self.documents {
            check_cancel(cancel)?;
            a.step()?;
            documents.push(DraftDocument {
                value: d.value.clone(),
                active: d.active,
            });
        }
        let mut transitions = Vec::new();
        a.reserve(&mut transitions, self.transitions.len())?;
        for t in &self.transitions {
            check_cancel(cancel)?;
            a.step()?;
            let mut records = Vec::new();
            a.reserve(&mut records, t.records.len())?;
            for r in &t.records {
                check_cancel(cancel)?;
                a.step()?;
                records.push(DraftRecord {
                    key: r.key,
                    name: r.name.clone(),
                    branches: r.branches,
                    rules: r.rules.clone(),
                    captured: r.captured,
                });
            }
            transitions.push(DraftTransition {
                key: t.key,
                document: t.document,
                records,
                setup: t.setup,
                root_captured: t.root_captured,
                next_record: t.next_record,
            });
        }
        Ok(Self {
            documents,
            transitions,
            next_transition: self.next_transition,
            transition_tickets: self.transition_tickets,
            record_tickets: self.record_tickets,
            published_transitions: a.copy(&self.published_transitions, cancel)?,
            published_records: a.copy(&self.published_records, cancel)?,
        })
    }
    pub(super) fn source_storage_bytes(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<usize, DomError> {
        let mut bytes = self
            .documents
            .capacity()
            .checked_mul(size_of::<DraftDocument>())
            .and_then(|n| {
                self.transitions
                    .capacity()
                    .checked_mul(size_of::<DraftTransition>())
                    .and_then(|m| n.checked_add(m))
            })
            .ok_or(DomError::RepresentationExhausted)?;
        bytes = bytes
            .checked_add(
                self.published_transitions
                    .capacity()
                    .checked_mul(size_of::<Option<DomViewTransitionId>>())
                    .ok_or(DomError::RepresentationExhausted)?,
            )
            .and_then(|n| {
                n.checked_add(
                    self.published_records
                        .capacity()
                        .checked_mul(size_of::<Option<DomViewTransitionRecordId>>())?,
                )
            })
            .ok_or(DomError::RepresentationExhausted)?;
        for t in &self.transitions {
            step()?;
            bytes = bytes
                .checked_add(
                    t.records
                        .capacity()
                        .checked_mul(size_of::<DraftRecord>())
                        .ok_or(DomError::RepresentationExhausted)?,
                )
                .ok_or(DomError::RepresentationExhausted)?;
            for record in &t.records {
                step()?;
                bytes = bytes
                    .checked_add(size_of::<surgeist_css::CssViewTransitionIdent>())
                    .and_then(|n| n.checked_add(record.name.retained_buffer_bytes()))
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        Ok(bytes)
    }
    fn phase_growth(&self) -> Result<usize, DomError> {
        let mut bytes = 0usize;
        for (length, capacity, width) in [
            (
                self.transition_tickets,
                self.published_transitions.capacity(),
                size_of::<Option<DomViewTransitionId>>(),
            ),
            (
                self.record_tickets,
                self.published_records.capacity(),
                size_of::<Option<DomViewTransitionRecordId>>(),
            ),
        ] {
            if length > capacity {
                bytes = bytes
                    .checked_add(
                        length
                            .checked_mul(width)
                            .ok_or(DomError::RepresentationExhausted)?,
                    )
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        Ok(bytes)
    }
    fn prepare_phase(&mut self) -> Result<(), DomError> {
        self.published_transitions
            .try_reserve_exact(self.transition_tickets - self.published_transitions.len())
            .map_err(|_| DomError::AllocationFailed)?;
        self.published_transitions
            .resize(self.transition_tickets, None);
        self.published_records
            .try_reserve_exact(self.record_tickets - self.published_records.len())
            .map_err(|_| DomError::AllocationFailed)?;
        self.published_records.resize(self.record_tickets, None);
        Ok(())
    }
    pub(super) fn acknowledge_publication(
        &mut self,
        state: &ViewTransitions,
        transitions: &[Option<DomViewTransitionId>],
        records: &[Option<DomViewTransitionRecordId>],
    ) {
        self.next_transition = state.next_transition;
        self.published_transitions.copy_from_slice(transitions);
        self.published_records.copy_from_slice(records);
        for (draft, actual) in self.transitions.iter_mut().zip(&state.transitions) {
            draft.next_record = actual.next_record;
        }
    }
    pub(super) fn new_formed(
        value: &ViewTransitions,
        initial: Option<&source::initial::Initial>,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let mut documents = Vec::new();
        source::initial::reserve(&mut documents, value.documents.len(), initial, cancel)?;
        for d in &value.documents {
            source::initial::step(initial, cancel)?;
            documents.push(DraftDocument {
                value: d.clone(),
                active: d.active.map(TransitionKey::Existing),
            });
        }
        let mut transitions = Vec::new();
        source::initial::reserve(&mut transitions, value.transitions.len(), initial, cancel)?;
        for t in &value.transitions {
            source::initial::step(initial, cancel)?;
            let mut records = Vec::new();
            source::initial::reserve(&mut records, t.records.len(), initial, cancel)?;
            for r in &t.records {
                source::initial::step(initial, cancel)?;
                records.push(DraftRecord {
                    key: RecordKey::Existing(r.id),
                    name: r.name.clone(),
                    branches: r.branches,
                    rules: r.rules.clone(),
                    captured: r.captured,
                });
            }
            transitions.push(DraftTransition {
                key: TransitionKey::Existing(t.id),
                document: t.document,
                records,
                setup: t.setup,
                root_captured: t.root_captured,
                next_record: t.next_record,
            });
        }
        Ok(Self {
            documents,
            transitions,
            next_transition: value.next_transition,
            transition_tickets: 0,
            record_tickets: 0,
            published_transitions: Vec::new(),
            published_records: Vec::new(),
        })
    }
    fn transition(
        &self,
        owner: &Arc<BatchIdentity>,
        store: DomOwnerId,
        operand: ViewTransitionOperand,
    ) -> Result<usize, DomError> {
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
                TransitionKey::Created(ticket.index)
            }
        };
        self.transitions
            .iter()
            .position(|t| t.key == key || matches!((key, t.key), (TransitionKey::Existing(id), TransitionKey::Created(ticket)) if self.published_transitions.get(ticket).copied().flatten() == Some(id)))
            .ok_or(match key {
                TransitionKey::Existing(_) => DomError::StaleViewTransition,
                TransitionKey::Created(_) => DomError::UnavailableCreation,
            })
    }
    fn record(
        &self,
        owner: &Arc<BatchIdentity>,
        store: DomOwnerId,
        operand: ViewTransitionRecordOperand,
    ) -> Result<(usize, usize), DomError> {
        let key = match operand {
            ViewTransitionRecordOperand::Existing(id) => {
                if id.transition.owner != store {
                    return Err(DomError::ForeignOwner);
                }
                RecordKey::Existing(id)
            }
            ViewTransitionRecordOperand::Created(ticket) => {
                if !Arc::ptr_eq(owner, &ticket.owner) {
                    return Err(DomError::ForeignBatch);
                }
                RecordKey::Created(ticket.index)
            }
        };
        for (t, transition) in self.transitions.iter().enumerate() {
            if let Some(r) = transition.records.iter().position(|r| r.key == key || matches!((key, r.key), (RecordKey::Existing(id), RecordKey::Created(ticket)) if self.published_records.get(ticket).copied().flatten() == Some(id))) {
                return Ok((t, r));
            }
        }
        Err(match key {
            RecordKey::Existing(_) => DomError::StaleViewTransitionRecord,
            RecordKey::Created(_) => DomError::UnavailableCreation,
        })
    }
    fn document(&self, document: DocumentId) -> Result<usize, DomError> {
        self.documents
            .iter()
            .position(|d| d.value.document == document)
            .ok_or(DomError::ViewTransitionsUninitialized)
    }
    fn checked_document(&self, t: usize, document: DocumentId) -> Result<(), DomError> {
        if self.transitions[t].document != document {
            return Err(DomError::WrongDocument);
        }
        Ok(())
    }
    fn mutable_topology(&self, t: usize, document: DocumentId) -> Result<(), DomError> {
        self.checked_document(t, document)?;
        let d = self.document(document)?;
        if self.transitions[t].setup || self.documents[d].active != Some(self.transitions[t].key) {
            return Err(DomError::InvalidViewTransitionState);
        }
        Ok(())
    }
    pub(super) fn finish(
        &self,
        owner: DomOwnerId,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<FinishedTransitions, DomError> {
        let mut transitions = Vec::new();
        let mut transition_ids = Vec::new();
        let mut record_ids = Vec::new();
        source::formation::reserve(&mut transitions, self.transitions.len(), budget, cancel)?;
        source::formation::reserve(&mut transition_ids, self.transition_tickets, budget, cancel)?;
        transition_ids.extend(self.published_transitions.iter().copied());
        transition_ids.resize(self.transition_tickets, None);
        source::formation::reserve(&mut record_ids, self.record_tickets, budget, cancel)?;
        record_ids.extend(self.published_records.iter().copied());
        record_ids.resize(self.record_tickets, None);
        let mut next_transition = self.next_transition;
        for t in &self.transitions {
            let id = match t.key {
                TransitionKey::Existing(id) => id,
                TransitionKey::Created(ticket) if transition_ids[ticket].is_some() => {
                    transition_ids[ticket].expect("published transition")
                }
                TransitionKey::Created(ticket) => {
                    let generation = next_transition;
                    next_transition = next_transition
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    let id = DomViewTransitionId { owner, generation };
                    transition_ids[ticket] = Some(id);
                    id
                }
            };
            let mut records = Vec::new();
            source::formation::reserve(&mut records, t.records.len(), budget, cancel)?;
            let mut next_record = t.next_record;
            for r in &t.records {
                if let Some(budget) = budget {
                    let mut payload = source::storage::Payload::default();
                    payload.add_bytes(size_of::<surgeist_css::CssViewTransitionIdent>())?;
                    payload.add_bytes(r.name.retained_buffer_bytes())?;
                    budget.retain_payload(payload, cancel)?;
                }
                let record = match r.key {
                    RecordKey::Existing(id) => id,
                    RecordKey::Created(ticket) if record_ids[ticket].is_some() => {
                        record_ids[ticket].expect("published record")
                    }
                    RecordKey::Created(ticket) => {
                        let generation = next_record;
                        next_record = next_record
                            .checked_add(1)
                            .ok_or(DomError::RepresentationExhausted)?;
                        let record = DomViewTransitionRecordId {
                            transition: id,
                            generation,
                        };
                        record_ids[ticket] = Some(record);
                        record
                    }
                };
                records.push(Record {
                    id: record,
                    name: r.name.clone(),
                    branches: r.branches,
                    rules: r.rules.clone(),
                    captured: r.captured,
                });
            }
            transitions.push(Transition {
                id,
                document: t.document,
                records,
                setup: t.setup,
                root_captured: t.root_captured,
                next_record,
            });
        }
        let mut documents = Vec::new();
        source::formation::reserve(&mut documents, self.documents.len(), budget, cancel)?;
        for d in &self.documents {
            let mut value = d.value.clone();
            value.active = match d.active {
                None => None,
                Some(TransitionKey::Existing(id)) => Some(id),
                Some(TransitionKey::Created(ticket)) => transition_ids[ticket],
            };
            documents.push(value);
        }
        Ok(FinishedTransitions {
            value: ViewTransitions {
                documents,
                transitions,
                next_transition,
            },
            transition_ids,
            record_ids,
        })
    }
}
pub(super) struct FinishedTransitions {
    pub value: ViewTransitions,
    pub transition_ids: Vec<Option<DomViewTransitionId>>,
    pub record_ids: Vec<Option<DomViewTransitionRecordId>>,
}
impl Draft {
    pub(super) fn prepare_transition_phase(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let extra = self.view_transitions.phase_growth()?;
        self.source_admit(extra, cancel)?;
        // Reserve the exact mandatory acknowledgement work before publication.
        for _ in 0..self.view_transitions.transition_tickets {
            self.source_step(cancel)?;
        }
        for _ in 0..self.view_transitions.record_tickets {
            self.source_step(cancel)?;
        }
        for _ in &self.view_transitions.transitions {
            self.source_step(cancel)?;
        }
        self.view_transitions.prepare_phase()?;
        self.source_admit(0, cancel)
    }
}
impl Draft {
    pub(super) fn apply_view_transition(
        &mut self,
        basis: crate::DomDocumentStamp,
        edit: &Edit,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<(), DomError> {
        if basis.store().owner() != self.store {
            return Err(DomError::ForeignOwner);
        }
        if basis.store().revision() != self.basis.revision() {
            return Err(DomError::ViewTransitionRevisionMismatch);
        }
        let document = basis.document();
        let key = self.resolve(document.into())?;
        if !matches!(self.node(key)?.kind, DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        if let Edit::Initialize { input } = edit {
            let DomPreparedViewTransitionSheet { sheet, revision } = input;
            if let Ok(d) = self.view_transitions.document(document) {
                return if self.view_transitions.documents[d].value.sheet.id == *sheet {
                    Ok(())
                } else {
                    Err(DomError::InvalidViewTransitionState)
                };
            }
            let bank = &self.view_transitions.documents;
            let admitted = self.source_reserve_amount::<DraftDocument>(
                bank.len(),
                bank.capacity(),
                1,
                cancelled,
            )?;
            source::reserve_bank(&mut self.view_transitions.documents, 1, admitted)?;
            self.source_admit(0, cancelled)?;
            self.view_transitions.documents.push(DraftDocument {
                value: DomDocumentViewTransitions {
                    document,
                    sheet: DomViewTransitionSheet {
                        id: sheet.clone(),
                        revision: revision.clone(),
                    },
                    active: None,
                    suppressed: false,
                    show: false,
                },
                active: None,
            });
            return Ok(());
        }
        let d = self.view_transitions.document(document)?;
        match edit {
            Edit::Initialize { .. } => unreachable!("initialization handled"),
            Edit::Begin { ticket } => {
                if self.view_transitions.documents[d].active.is_some() {
                    return Err(DomError::InvalidViewTransitionState);
                }
                let count = self
                    .view_transitions
                    .transitions
                    .len()
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(count, limits.transitions, DomResource::ViewTransitions)?;
                let (_, _, pseudos) = self.view_transitions.usage()?;
                admit(
                    pseudos
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                    limits.pseudos,
                    DomResource::ViewTransitionPseudos,
                )?;
                let bank = &self.view_transitions.transitions;
                let admitted = self.source_reserve_amount::<DraftTransition>(
                    bank.len(),
                    bank.capacity(),
                    1,
                    cancelled,
                )?;
                source::reserve_bank(&mut self.view_transitions.transitions, 1, admitted)?;
                self.source_admit(0, cancelled)?;
                let key = TransitionKey::Created(*ticket);
                self.view_transitions.transitions.push(DraftTransition {
                    key,
                    document,
                    records: Vec::new(),
                    setup: false,
                    root_captured: false,
                    next_record: 0,
                });
                self.view_transitions.transition_tickets =
                    self.view_transitions.transition_tickets.max(*ticket + 1);
                self.view_transitions.documents[d].active = Some(key);
            }
            Edit::Append {
                transition,
                name,
                ticket,
            } => {
                let t = self.view_transitions.transition(
                    &self.owner,
                    self.store,
                    transition.clone(),
                )?;
                self.view_transitions.mutable_topology(t, document)?;
                for record in &self.view_transitions.transitions[t].records {
                    self.admit_transition_name_read(&record.name, cancelled)?;
                    self.admit_transition_name_read(name, cancelled)?;
                    if record.name == *name {
                        return Err(DomError::DuplicateViewTransitionName);
                    }
                }
                let (_, records, _) = self.view_transitions.usage()?;
                admit(
                    records
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                    limits.records,
                    DomResource::ViewTransitionRecords,
                )?;
                self.admit_transition_name_read(name, cancelled)?;
                let units = self
                    .units
                    .checked_add(name.as_str().encode_utf16().count())
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                // This occurrence enters the Source domain only here. The
                // destination bank owns its inline Arc slot; lease only the
                // actual Ident header and UTF8 backing, not a CSSOM graph.
                let incoming_bytes = size_of::<surgeist_css::CssViewTransitionIdent>()
                    .checked_add(name.retained_buffer_bytes())
                    .ok_or(DomError::RepresentationExhausted)?;
                let incoming = source::formation::Formation::new(self, cancelled)?;
                if let Some(incoming) = &incoming {
                    incoming.retain_bytes(incoming_bytes, cancelled)?;
                }
                let name = name.clone();
                #[cfg(test)]
                append_observation::observe(self, 0, cancelled)?;
                let bank = &self.view_transitions.transitions[t].records;
                let admitted = self.source_reserve_amount::<DraftRecord>(
                    bank.len(),
                    bank.capacity(),
                    1,
                    cancelled,
                )?;
                source::reserve_bank(
                    &mut self.view_transitions.transitions[t].records,
                    1,
                    admitted,
                )?;
                self.source_admit(0, cancelled)?;
                self.view_transitions.transitions[t]
                    .records
                    .push(DraftRecord {
                        key: RecordKey::Created(*ticket),
                        name,
                        branches: DomViewTransitionBranches::Pending,
                        rules: None,
                        captured: [false; 4],
                    });
                // No fallible operation intervenes between canonical ownership
                // and releasing the temporary occurrence's admission.
                if let Some(incoming) = &incoming {
                    incoming.release_bytes(incoming_bytes);
                }
                self.view_transitions.record_tickets =
                    self.view_transitions.record_tickets.max(*ticket + 1);
                self.units = units;
            }
            Edit::Branches { record, branches } => {
                let (t, r) =
                    self.view_transitions
                        .record(&self.owner, self.store, record.clone())?;
                self.view_transitions.mutable_topology(t, document)?;
                if let Some(node) = branches.new_element() {
                    self.validate_transition_element(node, document)?;
                }
                if let Some(node) = branches.new_element()
                    && self.view_transitions.transitions[t]
                        .records
                        .iter()
                        .enumerate()
                        .any(|(other, value)| {
                            other != r && value.branches.new_element() == Some(node)
                        })
                {
                    return Err(DomError::InvalidViewTransitionState);
                }
                if let Some(rules) = &self.view_transitions.transitions[t].records[r].rules {
                    validate_rule_shape(*branches, &rules.associations)?;
                }
                self.view_transitions.transitions[t].records[r].branches = *branches;
            }
            Edit::Rules { record, input } => {
                let (t, r) =
                    self.view_transitions
                        .record(&self.owner, self.store, record.clone())?;
                self.view_transitions.checked_document(t, document)?;
                let value = &self.view_transitions.transitions[t].records[r];
                let binding = &self.view_transitions.documents[d].value.sheet;
                if input.revision.owner() != binding.id.owner() {
                    return Err(DomError::ViewTransitionCssom(
                        DomViewTransitionCssomError::ForeignOwner,
                    ));
                }
                if input.revision.value() < binding.revision.value()
                    || value
                        .rules
                        .as_ref()
                        .is_some_and(|old| input.revision.value() < old.revision.value())
                {
                    return Err(DomError::ViewTransitionCssom(
                        DomViewTransitionCssomError::StaleRevision,
                    ));
                }
                if input.sheet != binding.id {
                    return Err(DomError::ViewTransitionCssom(
                        DomViewTransitionCssomError::WrongSheet,
                    ));
                }
                validate_rule_shape(value.branches, &input.associations)?;
                self.view_transitions.transitions[t].records[r].rules =
                    Some(DomViewTransitionRules {
                        revision: input.revision.clone(),
                        associations: input.associations.clone(),
                    });
            }
            Edit::ClearRules(record) => {
                let (t, r) =
                    self.view_transitions
                        .record(&self.owner, self.store, record.clone())?;
                self.view_transitions.checked_document(t, document)?;
                self.view_transitions.transitions[t].records[r].rules = None;
            }
            Edit::Captured { subject, captured } => match *subject {
                DomViewTransitionSubject::Element(node) => {
                    let key = self.validate_transition_element(node, document)?;
                    let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
                        unreachable!("checked Element")
                    };
                    element.set_captured_in_view_transition(*captured);
                }
                DomViewTransitionSubject::Pseudo(id) => {
                    let t = self.view_transitions.transition(
                        &self.owner,
                        self.store,
                        id.transition.into(),
                    )?;
                    self.view_transitions.checked_document(t, document)?;
                    let transition = &mut self.view_transitions.transitions[t];
                    if let Some(record) = id.record {
                        let r = transition
                            .records
                            .iter_mut()
                            .find(|r| r.key == RecordKey::Existing(record))
                            .ok_or(DomError::StaleViewTransitionRecord)?;
                        if !transition.setup || !record_has_pseudo(r.branches, id.kind) {
                            return Err(DomError::StaleViewTransitionPseudo);
                        }
                        r.captured[flag_index(id.kind)] = *captured;
                    } else if id.kind == DomViewTransitionPseudoKind::Root {
                        transition.root_captured = *captured;
                    } else {
                        return Err(DomError::StaleViewTransitionPseudo);
                    }
                }
            },
            Edit::Suppression(suppressed) => {
                self.view_transitions.documents[d].value.suppressed = *suppressed
            }
            Edit::Setup(transition) => {
                let t = self.view_transitions.transition(
                    &self.owner,
                    self.store,
                    transition.clone(),
                )?;
                self.view_transitions.mutable_topology(t, document)?;
                let has_origin = self.node(key)?.children.iter().any(|child| {
                    matches!(
                        self.node(*child).map(|n| &n.kind),
                        Ok(DomNodeKind::Element(_))
                    )
                });
                if !has_origin {
                    return Err(DomError::ViewTransitionOriginUnavailable);
                }
                if self.view_transitions.transitions[t]
                    .records
                    .iter()
                    .any(|r| r.branches == DomViewTransitionBranches::Pending)
                {
                    return Err(DomError::InvalidViewTransitionState);
                }
                let mut count = self.view_transitions.usage()?.2;
                for r in &self.view_transitions.transitions[t].records {
                    check_cancel(cancelled)?;
                    count = count
                        .checked_add(
                            2 + usize::from(r.branches.has_old())
                                + usize::from(r.branches.new_element().is_some()),
                        )
                        .ok_or(DomError::RepresentationExhausted)?;
                }
                admit(count, limits.pseudos, DomResource::ViewTransitionPseudos)?;
                self.view_transitions.transitions[t].setup = true;
                self.view_transitions.documents[d].value.show = true;
            }
            Edit::Clear(transition) => {
                let t = self.view_transitions.transition(
                    &self.owner,
                    self.store,
                    transition.clone(),
                )?;
                self.view_transitions.checked_document(t, document)?;
                match self.view_transitions.documents[d].active {
                    None => return Ok(()),
                    Some(active) if active == self.view_transitions.transitions[t].key => {}
                    Some(_) => return Err(DomError::InvalidViewTransitionState),
                }
                for r in 0..self.view_transitions.transitions[t].records.len() {
                    check_cancel(cancelled)?;
                    if let Some(node) = self.view_transitions.transitions[t].records[r]
                        .branches
                        .new_element()
                    {
                        let key = match self.resolve(node.into()) {
                            Ok(key) => key,
                            Err(DomError::StaleNode) => continue,
                            Err(error) => return Err(error),
                        };
                        let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
                            return Err(DomError::WrongNodeKind);
                        };
                        element.set_captured_in_view_transition(false);
                    }
                }
                self.view_transitions.documents[d].active = None;
                self.view_transitions.documents[d].value.show = false;
            }
            Edit::Release(transition) => {
                let t = self.view_transitions.transition(
                    &self.owner,
                    self.store,
                    transition.clone(),
                )?;
                self.view_transitions.checked_document(t, document)?;
                if self.view_transitions.documents[d].active
                    == Some(self.view_transitions.transitions[t].key)
                {
                    return Err(DomError::InvalidViewTransitionState);
                }
                // Prepay the original UTF8-to-UTF16 reads while the records
                // remain in the graph; removal and subtraction are infallible.
                for record in &self.view_transitions.transitions[t].records {
                    self.admit_transition_name_read(&record.name, cancelled)?;
                }
                let removed = self.view_transitions.transitions.remove(t);
                for r in removed.records {
                    self.units -= r.name.as_str().encode_utf16().count();
                }
            }
        }
        Ok(())
    }
    fn admit_transition_name_read(
        &self,
        name: &surgeist_css::CssViewTransitionIdent,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Ordinary edits keep their original equality/count/cancellation order.
        // Source pays a header and every possible UTF8 byte before the original
        // checked producer equality or UTF16-length scan reads the operand.
        if self.source_limits.is_some() {
            self.source_step(cancel)?;
            for _ in 0..name.as_str().len() {
                self.source_step(cancel)?;
            }
        }
        Ok(())
    }
    fn validate_transition_element(
        &self,
        node: NodeId,
        document: DocumentId,
    ) -> Result<Key, DomError> {
        let key = self.resolve(node.into())?;
        let value = self.node(key)?;
        if value.document != Key::Existing(document.node().slot) {
            return Err(DomError::WrongDocument);
        }
        if !matches!(value.kind, DomNodeKind::Element(_)) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(key)
    }
}
impl DraftTransitions {
    pub(super) fn usage(&self) -> Result<(usize, usize, usize), DomError> {
        let mut records = 0usize;
        let mut pseudos = self.transitions.len();
        for t in &self.transitions {
            records = records
                .checked_add(t.records.len())
                .ok_or(DomError::RepresentationExhausted)?;
            if t.setup {
                for r in &t.records {
                    pseudos = pseudos
                        .checked_add(
                            2 + usize::from(r.branches.has_old())
                                + usize::from(r.branches.new_element().is_some()),
                        )
                        .ok_or(DomError::RepresentationExhausted)?;
                }
            }
        }
        Ok((self.transitions.len(), records, pseudos))
    }
}
fn validate_rule_shape(
    branches: DomViewTransitionBranches,
    rules: &DomViewTransitionRuleAssociations,
) -> Result<(), DomError> {
    use DomViewTransitionBranches as B;
    use DomViewTransitionImageRules as R;
    if matches!(
        (&branches, &rules.image_animation),
        (_, R::Empty)
            | (B::Old, R::Old(_))
            | (B::New(_), R::New(_))
            | (B::OldAndNew(_), R::OldThenNew { .. })
    ) {
        Ok(())
    } else {
        Err(DomError::InvalidViewTransitionState)
    }
}
impl ViewTransitions {
    pub(super) fn usage(&self) -> (usize, usize, usize) {
        let records = self.transitions.iter().map(|t| t.records.len()).sum();
        let pseudos = self
            .transitions
            .iter()
            .map(|t| {
                1 + if t.setup {
                    t.records
                        .iter()
                        .map(|r| {
                            2 + usize::from(r.branches.has_old())
                                + usize::from(r.branches.new_element().is_some())
                        })
                        .sum()
                } else {
                    0
                }
            })
            .sum();
        (self.transitions.len(), records, pseudos)
    }
    pub(super) fn changes(
        &self,
        old: &Self,
        changes: &mut Vec<DomChange>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        for d in &self.documents {
            if old.documents.iter().find(|o| o.document == d.document) != Some(d) {
                source::formation::reserve(changes, 1, budget, cancel)?;
                changes.push(DomChange::ViewTransitionDocument(d.document));
            }
        }
        for t in old.transitions.iter().chain(&self.transitions) {
            let before = old.transitions.iter().find(|o| o.id == t.id);
            let after = self.transitions.iter().find(|n| n.id == t.id);
            if before == after {
                continue;
            }
            if changes.contains(&DomChange::ViewTransition(t.id)) {
                continue;
            }
            source::formation::reserve(changes, 1, budget, cancel)?;
            changes.push(DomChange::ViewTransition(t.id));
            for r in before
                .into_iter()
                .flat_map(|t| &t.records)
                .chain(after.into_iter().flat_map(|t| &t.records))
            {
                let old_record = before.and_then(|t| t.records.iter().find(|old| old.id == r.id));
                let new_record = after.and_then(|t| t.records.iter().find(|new| new.id == r.id));
                let change = DomChange::ViewTransitionRecord(r.id);
                if old_record != new_record && !changes.contains(&change) {
                    source::formation::reserve(changes, 1, budget, cancel)?;
                    changes.push(change);
                }
            }
            if let (Some(before), Some(after)) = (before, after) {
                if before.root_captured != after.root_captured {
                    source::formation::reserve(changes, 1, budget, cancel)?;
                    changes.push(DomChange::ViewTransitionCaptured(
                        DomViewTransitionSubject::Pseudo(pseudo_id(
                            t.id,
                            None,
                            DomViewTransitionPseudoKind::Root,
                        )),
                    ));
                }
                for r in &after.records {
                    if let Some(old) = before.records.iter().find(|old| old.id == r.id) {
                        for kind in [
                            DomViewTransitionPseudoKind::Group,
                            DomViewTransitionPseudoKind::ImagePair,
                            DomViewTransitionPseudoKind::Old,
                            DomViewTransitionPseudoKind::New,
                        ] {
                            if r.captured[flag_index(kind)] != old.captured[flag_index(kind)] {
                                source::formation::reserve(changes, 1, budget, cancel)?;
                                changes.push(DomChange::ViewTransitionCaptured(
                                    DomViewTransitionSubject::Pseudo(pseudo_id(
                                        t.id,
                                        Some(r.id),
                                        kind,
                                    )),
                                ));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// Scalar observation at the actual incoming Append ownership boundary. The
// Source's ordinary private graph excludes a still-unregistered owned operand.
#[cfg(test)]
mod append_observation {
    use super::*;
    use std::cell::Cell;
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Growth {
        pub required: usize,
        pub limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static GROWTH: Cell<Option<Growth>> = const { Cell::new(None) };
        static CANCEL_AFTER_ACQUIRE: Cell<bool> = const { Cell::new(false) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        GROWTH.set(None);
        CANCEL_AFTER_ACQUIRE.set(false);
    }
    pub(super) fn cancel_at_occurrence() {
        start();
        CANCEL_AFTER_ACQUIRE.set(true);
    }
    pub(super) fn finish() -> Option<Growth> {
        ACTIVE.set(false);
        CANCEL_AFTER_ACQUIRE.set(false);
        GROWTH.take()
    }
    pub(super) fn observe(
        draft: &Draft,
        unregistered: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !ACTIVE.get() {
            return Ok(());
        }
        let Some(limits) = draft.source_limits else {
            return Ok(());
        };
        let required = draft
            .source_storage_bytes(cancel)?
            .checked_add(draft.source_other_bytes)
            .and_then(|n| n.checked_add(draft.source_formation_bytes.load(Ordering::Relaxed)))
            .and_then(|n| n.checked_add(unregistered))
            .ok_or(DomError::RepresentationExhausted)?;
        GROWTH.set(Some(Growth {
            required,
            limit: limits.retained_bytes,
        }));
        if CANCEL_AFTER_ACQUIRE.get() {
            cancel.store(true, Ordering::Release);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_css::{CssCustomIdent, CssViewTransitionIdent};
    use surgeist_cssom::{
        CssomContext, CssomEditGuard, CssomInputRole, CssomInputVersion, CssomLimits,
        CssomSheetInputs, CssomStore,
    };

    fn fixture() -> (Dom, DocumentId) {
        let mut dom = Dom::new(
            DomLimits::new(8, 256, 32, 0, 0, 8, 8192)
                .unwrap()
                .with_view_transitions(4, 4, 16)
                .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let document = b.create_document();
        let document = dom
            .apply(b, &AtomicBool::new(false))
            .unwrap()
            .document(&document)
            .unwrap();
        let mut css = CssomStore::new(CssomLimits::default(), CssomContext::default()).unwrap();
        let mut b = css
            .batch(CssomEditGuard::from_snapshot(&css.snapshot()))
            .unwrap();
        let sheet = b
            .create_parsed_sheet(
                "",
                CssomSheetInputs::constructed(
                    CssomInputVersion {
                        role: CssomInputRole::Document,
                        identity: "fixture".into(),
                        revision: 1,
                    },
                    "fixture".into(),
                    None,
                ),
            )
            .unwrap();
        let sheet = css.commit(b).unwrap().sheet(&sheet).unwrap().clone();
        let mut b = DomBatch::new();
        b.initialize_view_transitions(
            dom.snapshot().document(document).unwrap().stamp(),
            DomPreparedViewTransitionSheet::capture(
                &css.snapshot(),
                &sheet,
                DomViewTransitionCaptureLimits::new(16_384, 1_000_000),
                &AtomicBool::new(false),
            )
            .unwrap(),
        );
        dom.apply(b, &AtomicBool::new(false)).unwrap();
        (dom, document)
    }
    fn source_append_fixture() -> (Dom, DocumentId, DomViewTransitionId) {
        let (mut dom, document) = fixture();
        dom.set_limits(
            DomLimits::new(8, 8192, 32, 0, 0, 8, 8192)
                .unwrap()
                .with_view_transitions(4, 4, 16)
                .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let transition =
            batch.begin_view_transition(dom.snapshot().document(document).unwrap().stamp());
        let transition = dom
            .apply(batch, &AtomicBool::new(false))
            .unwrap()
            .view_transition(&transition)
            .unwrap()
            .unwrap();
        (dom, document, transition)
    }
    fn source_append_limits(bytes: usize) -> crate::DomSourceLimits {
        crate::DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096)
    }
    fn long_append_batch(
        dom: &Dom,
        document: DocumentId,
        transition: DomViewTransitionId,
    ) -> (
        DomBatch,
        CreatedViewTransitionRecord,
        Arc<CssViewTransitionIdent>,
    ) {
        let mut text = String::with_capacity(24_576);
        for _ in 0..4096 {
            text.push('源');
        }
        let ident =
            CssViewTransitionIdent::try_new(CssCustomIdent::try_new(text).unwrap()).unwrap();
        assert_eq!(ident.as_str().len(), 12_288);
        assert_eq!(ident.as_str().encode_utf16().count(), 4096);
        assert!(ident.retained_buffer_bytes() >= 24_576);
        let mut batch = DomBatch::new();
        let record = batch.append_view_transition_record(
            dom.snapshot().document(document).unwrap().stamp(),
            transition,
            ident,
        );
        let Command::ViewTransition {
            edit: Edit::Append { name, .. },
            ..
        } = &batch.commands[0]
        else {
            unreachable!()
        };
        let original = name.clone();
        (batch, record, original)
    }
    #[test]
    fn source_append_preadmits_long_utf8_name_before_owning_the_second_occurrence() {
        let cancel = AtomicBool::new(false);
        let (mut dom, document, transition) = source_append_fixture();
        let (batch, record, original) = long_append_batch(&dom, document, transition);
        append_observation::start();
        let completed = {
            let mut lease = dom
                .begin_source_commands(batch, source_append_limits(4_000_000), &cancel)
                .unwrap();
            let crate::DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
                panic!("scalar VT source completes without Style service");
            };
            done
        };
        let observed = append_observation::finish().expect("actual Append occurrence reached");
        assert!(observed.required <= observed.limit);
        let record = completed
            .commit()
            .view_transition_record(&record)
            .unwrap()
            .unwrap();
        assert_eq!(
            dom.snapshot()
                .view_transition_record(record)
                .unwrap()
                .name()
                .as_str(),
            original.as_str()
        );
        assert!(
            Arc::ptr_eq(
                &dom.state.view_transitions.transitions[0].records[0].name,
                &original
            ),
            "publication transfers the actual admitted backing"
        );

        let (mut dom, document, transition) = source_append_fixture();
        let before = dom.snapshot();
        let usage = dom.usage();
        let (batch, _, original) = long_append_batch(&dom, document, transition);
        append_observation::start();
        let refused = {
            let mut lease = dom
                .begin_source_commands(batch, source_append_limits(observed.required - 1), &cancel)
                .unwrap();
            lease.advance(&cancel)
        };
        let reached = append_observation::finish();
        assert!(
            matches!(
                refused,
                Err(DomError::ResourceLimit {
                    resource: DomResource::SourceRetainedBytes,
                    ..
                })
            ),
            "{refused:?}"
        );
        assert!(
            reached.is_none(),
            "over-limit actual Append occurrence must not be acquired: {reached:?}"
        );
        assert_eq!(
            Arc::strong_count(&original),
            1,
            "aborted source retains no incoming name"
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert!(dom.pending_source_edit().is_none());
    }

    #[test]
    fn source_append_exact_ident_capacity_is_admitted_before_later_record_bank_growth() {
        let cancel = AtomicBool::new(false);
        let (dom, document, transition) = source_append_fixture();
        for one_short in [false, true] {
            let (batch, _, original) = long_append_batch(&dom, document, transition);
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                source_append_limits(4_000_000),
                &cancel,
            )
            .unwrap();
            let existing = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
            let incoming = size_of::<CssViewTransitionIdent>() + original.retained_buffer_bytes();
            let limit = existing + incoming - usize::from(one_short);
            draft.source_limits = Some(source_append_limits(limit));
            let live = draft.source_formation_bytes.clone();
            let Command::ViewTransition { basis, edit } = &batch.commands[0] else {
                unreachable!()
            };
            append_observation::start();
            let result = draft.apply_view_transition(*basis, edit, dom.limits(), &cancel);
            let reached = append_observation::finish();
            assert!(
                matches!(
                    result,
                    Err(DomError::ResourceLimit {
                        resource: DomResource::SourceRetainedBytes,
                        ..
                    })
                ),
                "{result:?}"
            );
            if one_short {
                assert!(
                    reached.is_none(),
                    "no incoming occurrence on one-short refusal"
                );
            } else {
                let reached = reached.expect("exact Ident bytes admit the real Arc clone");
                assert_eq!(reached.required, limit);
                assert_eq!(reached.limit, limit);
                // The independent record Vec still needs its own actual bank;
                // its later refusal must release the earlier name lease.
            }
            assert_eq!(live.load(Ordering::Relaxed), 0);
            assert!(draft.view_transitions.transitions[0].records.is_empty());
            assert_eq!(
                Arc::strong_count(&original),
                2,
                "only caller and batch own the name"
            );
        }
    }

    #[test]
    fn source_append_cancelled_after_name_acquisition_releases_the_real_temporary() {
        let cancel = AtomicBool::new(false);
        let (dom, document, transition) = source_append_fixture();
        let before = dom.snapshot();
        let (batch, _, original) = long_append_batch(&dom, document, transition);
        let (mut draft, _, _) = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            source_append_limits(4_000_000),
            &cancel,
        )
        .unwrap();
        let live = draft.source_formation_bytes.clone();
        let units = draft.units;
        let Command::ViewTransition { basis, edit } = &batch.commands[0] else {
            unreachable!()
        };
        append_observation::cancel_at_occurrence();
        let result = draft.apply_view_transition(*basis, edit, dom.limits(), &cancel);
        let actual = append_observation::finish().expect("actual admitted Arc occurrence acquired");
        assert!(actual.required <= actual.limit);
        assert_eq!(result, Err(DomError::Cancelled));
        assert_eq!(live.load(Ordering::Relaxed), 0);
        assert_eq!(draft.units, units);
        assert!(draft.view_transitions.transitions[0].records.is_empty());
        assert_eq!(Arc::strong_count(&original), 2);
        assert_eq!(*dom.state, *before.state);
    }

    #[test]
    fn source_name_reads_obey_original_work_cancellation_and_duplicate_rollback() {
        let cancel = AtomicBool::new(false);
        let (mut dom, document, transition) = source_append_fixture();
        let (batch, _, original) = long_append_batch(&dom, document, transition);
        let before_units = dom.usage().string_code_units();
        dom.apply(batch, &cancel).unwrap();
        assert_eq!(
            dom.usage().string_code_units(),
            before_units + 4096,
            "native canonical units count UTF16, independently of UTF8 capacity"
        );
        let before = dom.snapshot();
        for stop in [0, 1, 2] {
            let (batch, _, incoming) = long_append_batch(&dom, document, transition);
            let (mut draft, _, _) = Draft::new_source(
                &dom,
                batch.owner.clone(),
                &batch.commands,
                source_append_limits(4_000_000),
                &cancel,
            )
            .unwrap();
            let spent = draft.source_work.load(Ordering::Relaxed);
            let limit = spent + incoming.as_str().len() as u64;
            if stop == 0 {
                draft.source_limits = Some(crate::DomSourceLimits::new(
                    4_000_000, 1_000_000, limit, 64, 4096, 4096,
                ));
            }
            cancel.store(stop == 1, Ordering::Release);
            let Command::ViewTransition { basis, edit } = &batch.commands[0] else {
                unreachable!()
            };
            append_observation::start();
            let result = draft.apply_view_transition(*basis, edit, dom.limits(), &cancel);
            assert!(
                append_observation::finish().is_none(),
                "no new name ownership before comparison succeeds"
            );
            cancel.store(false, Ordering::Release);
            match stop {
                0 => {
                    assert_eq!(result, Err(DomError::SourceWorkLimit { limit }));
                    assert_eq!(draft.source_work.load(Ordering::Relaxed), limit);
                }
                1 => assert_eq!(result, Err(DomError::Cancelled)),
                _ => assert_eq!(result, Err(DomError::DuplicateViewTransitionName)),
            }
            assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
            assert_eq!(draft.view_transitions.transitions[0].records.len(), 1);
            drop(draft);
            assert_eq!(Arc::strong_count(&incoming), 2);
            assert_eq!(*dom.state, *before.state);
        }
        assert_eq!(
            dom.state.view_transitions.transitions[0].records[0]
                .name
                .as_str(),
            original.as_str()
        );
        let mut clear = DomBatch::new();
        let stamp = dom.snapshot().document(document).unwrap().stamp();
        clear.clear_view_transition_state(stamp, transition);
        clear.release_view_transition(stamp, transition);
        let mut lease = dom
            .begin_source_commands(clear, source_append_limits(4_000_000), &cancel)
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel).unwrap(),
            crate::DomSourceProgress::Complete(_)
        ));
        drop(lease);
        assert_eq!(dom.usage().string_code_units(), before_units);
    }

    #[test]
    fn transition_generation_exhaustion_and_failed_publication_preserve_every_owned_state() {
        let (mut dom, document) = fixture();
        Arc::get_mut(&mut dom.state)
            .unwrap()
            .view_transitions
            .next_transition = u64::MAX;
        let old = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.begin_view_transition(old.document(document).unwrap().stamp());
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert_eq!(*dom.state, *old.state);
        assert_eq!(dom.usage(), usage);
        assert_eq!(dom.snapshot().stamp(), old.stamp());
    }
    #[test]
    fn record_generation_exhaustion_never_installs_a_name_or_consumes_live_history() {
        let (mut dom, document) = fixture();
        let mut b = DomBatch::new();
        let ticket = b.begin_view_transition(dom.snapshot().document(document).unwrap().stamp());
        let t = dom
            .apply(b, &AtomicBool::new(false))
            .unwrap()
            .view_transition(&ticket)
            .unwrap()
            .unwrap();
        Arc::get_mut(&mut dom.state)
            .unwrap()
            .view_transitions
            .transitions[0]
            .next_record = u64::MAX;
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut b = DomBatch::new();
        b.append_view_transition_record(
            before.document(document).unwrap().stamp(),
            t,
            CssViewTransitionIdent::try_new(CssCustomIdent::try_new("participant").unwrap())
                .unwrap(),
        );
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
    }
    #[test]
    fn cancellation_after_prepare_and_revision_exhaustion_publish_no_transition_or_slot_change() {
        let (mut dom, document) = fixture();
        let old = dom.snapshot();
        let mut b = DomBatch::new();
        b.begin_view_transition(old.document(document).unwrap().stamp());
        let cancel = AtomicBool::new(false);
        let prepared = dom.prepare(b, &cancel).unwrap();
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *old.state);
        dom.revision = DomRevision(u64::MAX);
        let mut b = DomBatch::new();
        b.set_view_transition_rendering_suppression(
            dom.snapshot().document(document).unwrap().stamp(),
            true,
        );
        assert_eq!(
            dom.apply(b, &AtomicBool::new(false)).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *old.state);
    }
    #[test]
    fn transient_transition_and_record_tickets_consume_no_generation_and_released_ids_never_revive()
    {
        let (mut dom, document) = fixture();
        let before = dom.snapshot();
        let generation = dom.state.view_transitions.next_transition;
        let mut b = DomBatch::new();
        let stamp = before.document(document).unwrap().stamp();
        let t = b.begin_view_transition(stamp);
        let r = b.append_view_transition_record(
            stamp,
            &t,
            CssViewTransitionIdent::try_new(CssCustomIdent::try_new("temporary").unwrap()).unwrap(),
        );
        b.clear_view_transition_state(stamp, &t);
        b.release_view_transition(stamp, &t);
        let report = dom.apply(b, &AtomicBool::new(false)).unwrap();
        assert_eq!(report.view_transition(&t).unwrap(), None);
        assert_eq!(report.view_transition_record(&r).unwrap(), None);
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.state.view_transitions.next_transition, generation);
        let mut b = DomBatch::new();
        let t = b.begin_view_transition(stamp);
        let first = dom
            .apply(b, &AtomicBool::new(false))
            .unwrap()
            .view_transition(&t)
            .unwrap()
            .unwrap();
        assert_eq!(first.generation, generation);
        let stamp = dom.snapshot().document(document).unwrap().stamp();
        let mut b = DomBatch::new();
        b.clear_view_transition_state(stamp, first);
        b.release_view_transition(stamp, first);
        dom.apply(b, &AtomicBool::new(false)).unwrap();
        let mut b = DomBatch::new();
        let t = b.begin_view_transition(dom.snapshot().document(document).unwrap().stamp());
        let next = dom
            .apply(b, &AtomicBool::new(false))
            .unwrap()
            .view_transition(&t)
            .unwrap()
            .unwrap();
        assert_ne!(first, next);
        assert_eq!(
            dom.snapshot().view_transition(first).unwrap_err(),
            DomError::StaleViewTransition
        );
    }
}
