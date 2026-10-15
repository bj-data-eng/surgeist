use super::*;
use crate::view_transition::*;
use surgeist_cssom::{CssomParent, CssomRuleData, CssomSnapshot};

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
    name: surgeist_css::CssViewTransitionIdent,
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
}
impl DraftTransitions {
    pub(super) fn new(value: &ViewTransitions) -> Result<Self, DomError> {
        let mut documents = Vec::new();
        reserve(&mut documents, value.documents.len())?;
        documents.extend(value.documents.iter().map(|d| DraftDocument {
            value: d.clone(),
            active: d.active.map(TransitionKey::Existing),
        }));
        let mut transitions = Vec::new();
        reserve(&mut transitions, value.transitions.len())?;
        for t in &value.transitions {
            let mut records = Vec::new();
            reserve(&mut records, t.records.len())?;
            records.extend(t.records.iter().map(|r| DraftRecord {
                key: RecordKey::Existing(r.id),
                name: r.name.clone(),
                branches: r.branches,
                rules: r.rules.clone(),
                captured: r.captured,
            }));
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
            .position(|t| t.key == key)
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
            if let Some(r) = transition.records.iter().position(|r| r.key == key) {
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
    pub(super) fn finish(self, owner: DomOwnerId) -> Result<FinishedTransitions, DomError> {
        let mut transitions = Vec::new();
        let mut transition_ids = Vec::new();
        let mut record_ids = Vec::new();
        reserve(&mut transitions, self.transitions.len())?;
        reserve(&mut transition_ids, self.transition_tickets)?;
        transition_ids.resize(self.transition_tickets, None);
        reserve(&mut record_ids, self.record_tickets)?;
        record_ids.resize(self.record_tickets, None);
        let mut next_transition = self.next_transition;
        for t in self.transitions {
            let id = match t.key {
                TransitionKey::Existing(id) => id,
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
            reserve(&mut records, t.records.len())?;
            let mut next_record = t.next_record;
            for r in t.records {
                let record = match r.key {
                    RecordKey::Existing(id) => id,
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
                    name: r.name,
                    branches: r.branches,
                    rules: r.rules,
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
        reserve(&mut documents, self.documents.len())?;
        for mut d in self.documents {
            d.value.active = match d.active {
                None => None,
                Some(TransitionKey::Existing(id)) => Some(id),
                Some(TransitionKey::Created(ticket)) => transition_ids[ticket],
            };
            documents.push(d.value);
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
    pub(super) fn apply_view_transition(
        &mut self,
        basis: crate::DomDocumentStamp,
        edit: Edit,
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
        if let Edit::Initialize { snapshot, sheet } = edit {
            snapshot.sheet(&sheet).map_err(cssom_error)?;
            if let Ok(d) = self.view_transitions.document(document) {
                return if self.view_transitions.documents[d].value.sheet.id == sheet {
                    Ok(())
                } else {
                    Err(DomError::InvalidViewTransitionState)
                };
            }
            reserve(&mut self.view_transitions.documents, 1)?;
            self.view_transitions.documents.push(DraftDocument {
                value: DomDocumentViewTransitions {
                    document,
                    sheet: DomViewTransitionSheet {
                        id: sheet,
                        revision: snapshot.revision().clone(),
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
                reserve(&mut self.view_transitions.transitions, 1)?;
                let key = TransitionKey::Created(ticket);
                self.view_transitions.transitions.push(DraftTransition {
                    key,
                    document,
                    records: Vec::new(),
                    setup: false,
                    root_captured: false,
                    next_record: 0,
                });
                self.view_transitions.transition_tickets =
                    self.view_transitions.transition_tickets.max(ticket + 1);
                self.view_transitions.documents[d].active = Some(key);
            }
            Edit::Append {
                transition,
                name,
                ticket,
            } => {
                let t = self
                    .view_transitions
                    .transition(&self.owner, self.store, transition)?;
                self.view_transitions.mutable_topology(t, document)?;
                if self.view_transitions.transitions[t]
                    .records
                    .iter()
                    .any(|r| r.name == name)
                {
                    return Err(DomError::DuplicateViewTransitionName);
                }
                let (_, records, _) = self.view_transitions.usage()?;
                admit(
                    records
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                    limits.records,
                    DomResource::ViewTransitionRecords,
                )?;
                let units = self
                    .units
                    .checked_add(name.as_str().encode_utf16().count())
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                reserve(&mut self.view_transitions.transitions[t].records, 1)?;
                self.view_transitions.transitions[t]
                    .records
                    .push(DraftRecord {
                        key: RecordKey::Created(ticket),
                        name,
                        branches: DomViewTransitionBranches::Pending,
                        rules: None,
                        captured: [false; 4],
                    });
                self.view_transitions.record_tickets =
                    self.view_transitions.record_tickets.max(ticket + 1);
                self.units = units;
            }
            Edit::Branches { record, branches } => {
                let (t, r) = self
                    .view_transitions
                    .record(&self.owner, self.store, record)?;
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
                    validate_rule_shape(branches, &rules.associations)?;
                }
                self.view_transitions.transitions[t].records[r].branches = branches;
            }
            Edit::Rules {
                record,
                snapshot,
                rules,
            } => {
                let (t, r) = self
                    .view_transitions
                    .record(&self.owner, self.store, record)?;
                self.view_transitions.checked_document(t, document)?;
                let value = &self.view_transitions.transitions[t].records[r];
                let binding = &self.view_transitions.documents[d].value.sheet;
                if snapshot.owner() != binding.id.owner() {
                    return Err(DomError::ViewTransitionCssom(
                        DomViewTransitionCssomError::ForeignOwner,
                    ));
                }
                if snapshot.revision().value() < binding.revision.value()
                    || value
                        .rules
                        .as_ref()
                        .is_some_and(|old| snapshot.revision().value() < old.revision.value())
                {
                    return Err(DomError::ViewTransitionCssom(
                        DomViewTransitionCssomError::StaleRevision,
                    ));
                }
                validate_rules(&snapshot, &binding.id, value.branches, &rules)?;
                self.view_transitions.transitions[t].records[r].rules =
                    Some(DomViewTransitionRules {
                        revision: snapshot.revision().clone(),
                        associations: rules,
                    });
            }
            Edit::ClearRules(record) => {
                let (t, r) = self
                    .view_transitions
                    .record(&self.owner, self.store, record)?;
                self.view_transitions.checked_document(t, document)?;
                self.view_transitions.transitions[t].records[r].rules = None;
            }
            Edit::Captured { subject, captured } => match subject {
                DomViewTransitionSubject::Element(node) => {
                    let key = self.validate_transition_element(node, document)?;
                    let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
                        unreachable!("checked Element")
                    };
                    element.set_captured_in_view_transition(captured);
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
                        r.captured[flag_index(id.kind)] = captured;
                    } else if id.kind == DomViewTransitionPseudoKind::Root {
                        transition.root_captured = captured;
                    } else {
                        return Err(DomError::StaleViewTransitionPseudo);
                    }
                }
            },
            Edit::Suppression(suppressed) => {
                self.view_transitions.documents[d].value.suppressed = suppressed
            }
            Edit::Setup(transition) => {
                let t = self
                    .view_transitions
                    .transition(&self.owner, self.store, transition)?;
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
                let t = self
                    .view_transitions
                    .transition(&self.owner, self.store, transition)?;
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
                let t = self
                    .view_transitions
                    .transition(&self.owner, self.store, transition)?;
                self.view_transitions.checked_document(t, document)?;
                if self.view_transitions.documents[d].active
                    == Some(self.view_transitions.transitions[t].key)
                {
                    return Err(DomError::InvalidViewTransitionState);
                }
                let removed = self.view_transitions.transitions.remove(t);
                for r in removed.records {
                    self.units -= r.name.as_str().encode_utf16().count();
                }
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
    fn usage(&self) -> Result<(usize, usize, usize), DomError> {
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
fn cssom_error(error: surgeist_cssom::CssomError) -> DomError {
    use DomViewTransitionCssomError::*;
    DomError::ViewTransitionCssom(match error {
        surgeist_cssom::CssomError::ForeignOwner => ForeignOwner,
        _ => MissingObject,
    })
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
fn validate_rules(
    snapshot: &CssomSnapshot,
    sheet: &surgeist_cssom::CssomSheetId,
    branches: DomViewTransitionBranches,
    rules: &DomViewTransitionRuleAssociations,
) -> Result<(), DomError> {
    use DomViewTransitionCssomError as E;
    validate_rule_shape(branches, rules)?;
    let list = snapshot
        .rules(snapshot.sheet(sheet).map_err(cssom_error)?.rules())
        .map_err(cssom_error)?;
    let (old, new) = match &rules.image_animation {
        DomViewTransitionImageRules::Empty => (None, None),
        DomViewTransitionImageRules::Old(id) => (Some(id), None),
        DomViewTransitionImageRules::New(id) => (None, Some(id)),
        DomViewTransitionImageRules::OldThenNew { old, new } => (Some(old), Some(new)),
    };
    let ids = [
        rules.group_keyframes.as_ref(),
        rules.group_animation_name.as_ref(),
        rules.group_styles.as_ref(),
        rules.image_pair_isolation.as_ref(),
        old,
        new,
    ];
    for (index, id) in ids.iter().enumerate() {
        let Some(id) = id else { continue };
        let rule = snapshot.rule(id).map_err(cssom_error)?;
        if !matches!(
            (index, rule.data()),
            (0, CssomRuleData::Keyframes { .. }) | (1..=5, CssomRuleData::Style { .. })
        ) {
            return Err(DomError::ViewTransitionCssom(E::WrongKind));
        }
        if rule.parent() != &CssomParent::Sheet(sheet.clone()) || !list.contains(id) {
            return Err(DomError::ViewTransitionCssom(E::WrongSheet));
        }
        if ids[..index].iter().flatten().any(|other| other == id) {
            return Err(DomError::ViewTransitionCssom(E::DuplicateRule));
        }
    }
    if let (Some(old), Some(new)) = (old, new)
        && list.iter().position(|id| id == old) >= list.iter().position(|id| id == new)
    {
        return Err(DomError::ViewTransitionCssom(E::RuleOrder));
    }
    Ok(())
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
    pub(super) fn changes(&self, old: &Self, changes: &mut Vec<DomChange>) -> Result<(), DomError> {
        for d in &self.documents {
            if old.documents.iter().find(|o| o.document == d.document) != Some(d) {
                reserve(changes, 1)?;
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
            reserve(changes, 1)?;
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
                    reserve(changes, 1)?;
                    changes.push(change);
                }
            }
            if let (Some(before), Some(after)) = (before, after) {
                if before.root_captured != after.root_captured {
                    reserve(changes, 1)?;
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
                                reserve(changes, 1)?;
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
            &css.snapshot(),
            &sheet,
        );
        dom.apply(b, &AtomicBool::new(false)).unwrap();
        (dom, document)
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
