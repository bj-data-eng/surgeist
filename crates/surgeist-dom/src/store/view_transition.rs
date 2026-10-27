use super::*;
use crate::view_transition::*;
mod membership;
mod storage;
pub(crate) use membership::TransitionMembership;
pub(in crate::store) use storage::RecordKey;
use storage::{DraftDocument, DraftTransition, TransitionKey};
pub(super) use storage::{DraftTransitions, TransitionAck, TransitionCustody, TransitionUndo};

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
        let document_key = self.resolve(document.into())?;
        if !matches!(
            self.node(document_key)?.kind(),
            transaction::KindRead::Document(_)
        ) {
            return Err(DomError::WrongNodeKind);
        }
        let mut next = self.view_transitions.clone();
        let registry = self.custody_registry().clone();
        let mut admit_storage = |usage| self.admit_custody_usage(usage, cancelled);
        let mut units = self.units;
        if let Edit::Initialize { input } = edit {
            if let Ok(current) = next.document(document) {
                return if current.value.sheet.id == input.sheet {
                    Ok(())
                } else {
                    Err(DomError::InvalidViewTransitionState)
                };
            }
            let ordinal = next.next_document;
            next.next_document = ordinal
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            next.set_document(
                DraftDocument {
                    value: DomDocumentViewTransitions {
                        init_ordinal: ordinal,
                        document,
                        sheet: DomViewTransitionSheet {
                            id: input.sheet.clone(),
                            revision: input.revision.clone(),
                        },
                        active: None,
                        suppressed: false,
                        show: false,
                    },
                    active: None,
                },
                &registry,
                &mut admit_storage,
            )?;
        } else {
            let mut doc = next.document(document)?;
            match edit {
                Edit::Initialize { .. } => unreachable!("initialization handled"),
                Edit::Begin { ticket } => {
                    if doc.active.is_some() {
                        return Err(DomError::InvalidViewTransitionState);
                    }
                    let count = next
                        .counts
                        .0
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    let pseudos = next
                        .counts
                        .2
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(count, limits.transitions, DomResource::ViewTransitions)?;
                    admit(pseudos, limits.pseudos, DomResource::ViewTransitionPseudos)?;
                    let key = TransitionKey::Created(*ticket);
                    next.set(
                        DraftTransition::empty(key, document),
                        &registry,
                        &mut admit_storage,
                    )?;
                    next.transition_tickets = next.transition_tickets.max(
                        ticket
                            .checked_add(1)
                            .ok_or(DomError::RepresentationExhausted)?,
                    );
                    next.counts.0 = count;
                    next.counts.2 = pseudos;
                    doc.active = Some(key);
                    next.set_document(doc, &registry, &mut admit_storage)?;
                }
                Edit::Append {
                    transition,
                    name,
                    ticket,
                } => {
                    let key = next.transition(&self.owner, self.store, transition.clone())?;
                    next.mutable_topology(key, document)?;
                    let mut value = next.get(key)?;
                    if value
                        .name_owner(name, &mut || self.source_step(cancelled))?
                        .is_some()
                    {
                        return Err(DomError::DuplicateViewTransitionName);
                    }
                    let records = next
                        .counts
                        .1
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(records, limits.records, DomResource::ViewTransitionRecords)?;
                    self.admit_transition_name_read(name, cancelled)?;
                    let name_units = name.as_str().encode_utf16().count();
                    units = units
                        .checked_add(name_units)
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(units, limits.units, DomResource::StringCodeUnits)?;
                    value.append_record(
                        *ticket,
                        name,
                        &registry,
                        &mut admit_storage,
                        &mut || self.source_step(cancelled),
                        || {
                            #[cfg(test)]
                            append_observation::observe(self, 0, cancelled)?;
                            check_cancel(cancelled)
                        },
                    )?;
                    check_cancel(cancelled)?;
                    value.name_units = value
                        .name_units
                        .checked_add(name_units)
                        .ok_or(DomError::RepresentationExhausted)?;
                    next.record_owner(*ticket, key, &registry, &mut admit_storage)?;
                    next.set(value, &registry, &mut admit_storage)?;
                    next.record_tickets = next.record_tickets.max(
                        ticket
                            .checked_add(1)
                            .ok_or(DomError::RepresentationExhausted)?,
                    );
                    next.counts.1 = records;
                }
                Edit::Branches { record, branches } => {
                    let (key, record_key) = next.record(&self.owner, self.store, record.clone())?;
                    next.mutable_topology(key, document)?;
                    let mut value = next.get(key)?;
                    if let Some(node) = branches.new_element() {
                        self.validate_transition_element(node, document)?;
                    }
                    if let Some(node) = branches.new_element()
                        && value
                            .element_owner(node, &mut || self.source_step(cancelled))?
                            .is_some_and(|owner| owner != record_key)
                    {
                        return Err(DomError::InvalidViewTransitionState);
                    }
                    if let Some(rules) = value.record(record_key)?.rules {
                        validate_rule_shape(*branches, &rules.associations)?;
                    }
                    value.change_record(
                        record_key,
                        &registry,
                        &mut admit_storage,
                        &mut || self.source_step(cancelled),
                        |read| {
                            let mut record = read.owned();
                            record.branches = *branches;
                            record
                        },
                    )?;
                    next.set(value, &registry, &mut admit_storage)?;
                }
                Edit::Rules { record, input } => {
                    let (key, record_key) = next.record(&self.owner, self.store, record.clone())?;
                    next.checked_document(key, document)?;
                    let mut value = next.get(key)?;
                    if doc.value.sheet.revision.owner() != input.revision.owner() {
                        return Err(DomError::ViewTransitionCssom(
                            DomViewTransitionCssomError::ForeignOwner,
                        ));
                    }
                    if doc.value.sheet.id != input.sheet {
                        return Err(DomError::ViewTransitionCssom(
                            DomViewTransitionCssomError::WrongSheet,
                        ));
                    }
                    if input.revision.value() < doc.value.sheet.revision.value() {
                        return Err(DomError::ViewTransitionCssom(
                            DomViewTransitionCssomError::StaleRevision,
                        ));
                    }
                    validate_rule_shape(value.record(record_key)?.branches, &input.associations)?;
                    value.change_record(
                        record_key,
                        &registry,
                        &mut admit_storage,
                        &mut || self.source_step(cancelled),
                        |read| {
                            let mut record = read.owned();
                            record.rules = Some(DomViewTransitionRules {
                                revision: input.revision.clone(),
                                associations: input.associations.clone(),
                            });
                            record
                        },
                    )?;
                    next.set(value, &registry, &mut admit_storage)?;
                    doc.value.sheet.revision = input.revision.clone();
                    next.set_document(doc, &registry, &mut admit_storage)?;
                }
                Edit::ClearRules(record) => {
                    let (key, record_key) = next.record(&self.owner, self.store, record.clone())?;
                    next.checked_document(key, document)?;
                    let mut value = next.get(key)?;
                    value.change_record(
                        record_key,
                        &registry,
                        &mut admit_storage,
                        &mut || self.source_step(cancelled),
                        |read| {
                            let mut record = read.owned();
                            record.rules = None;
                            record
                        },
                    )?;
                    next.set(value, &registry, &mut admit_storage)?;
                }
                Edit::Captured { subject, captured } => match *subject {
                    DomViewTransitionSubject::Element(node) => {
                        let key = self.validate_transition_element(node, document)?;
                        self.with_element_fields(key, cancelled, |element| {
                            element.set_captured_in_view_transition(*captured)
                        })?;
                    }
                    DomViewTransitionSubject::Pseudo(id) => {
                        let key = next.transition(&self.owner, self.store, id.transition.into())?;
                        next.checked_document(key, document)?;
                        let mut value = next.get(key)?;
                        if let Some(record) = id.record {
                            let record_key = RecordKey::Existing(record);
                            if !value.setup
                                || !record_has_pseudo(value.record(record_key)?.branches, id.kind)
                            {
                                return Err(DomError::StaleViewTransitionPseudo);
                            }
                            value.change_record(
                                record_key,
                                &registry,
                                &mut admit_storage,
                                &mut || self.source_step(cancelled),
                                |read| {
                                    let mut record = read.owned();
                                    record.captured[flag_index(id.kind)] = *captured;
                                    record
                                },
                            )?;
                        } else if id.kind == DomViewTransitionPseudoKind::Root {
                            value.root_captured = *captured;
                        } else {
                            return Err(DomError::StaleViewTransitionPseudo);
                        }
                        next.set(value, &registry, &mut admit_storage)?;
                    }
                },
                Edit::Suppression(suppressed) => {
                    doc.value.suppressed = *suppressed;
                    next.set_document(doc, &registry, &mut admit_storage)?;
                }
                Edit::Setup(transition) => {
                    let key = next.transition(&self.owner, self.store, transition.clone())?;
                    next.mutable_topology(key, document)?;
                    let mut value = next.get(key)?;
                    let has_origin = self.node(document_key)?.children().iter().any(|child| {
                        matches!(
                            self.node(child).map(|node| node.kind()),
                            Ok(transaction::KindRead::Element(_))
                        )
                    });
                    if !has_origin {
                        return Err(DomError::ViewTransitionOriginUnavailable);
                    }
                    let mut added = 0usize;
                    for record in value.records() {
                        self.source_step(cancelled)?;
                        if record.branches == DomViewTransitionBranches::Pending {
                            return Err(DomError::InvalidViewTransitionState);
                        }
                        added = added
                            .checked_add(
                                2 + usize::from(record.branches.has_old())
                                    + usize::from(record.branches.new_element().is_some()),
                            )
                            .ok_or(DomError::RepresentationExhausted)?;
                    }
                    let pseudos = next
                        .counts
                        .2
                        .checked_add(added)
                        .ok_or(DomError::RepresentationExhausted)?;
                    admit(pseudos, limits.pseudos, DomResource::ViewTransitionPseudos)?;
                    value.setup = true;
                    value.pseudos = value
                        .pseudos
                        .checked_add(added)
                        .ok_or(DomError::RepresentationExhausted)?;
                    next.set(value, &registry, &mut admit_storage)?;
                    next.counts.2 = pseudos;
                    doc.value.show = true;
                    next.set_document(doc, &registry, &mut admit_storage)?;
                }
                Edit::Clear(transition) => {
                    let key = next.transition(&self.owner, self.store, transition.clone())?;
                    next.checked_document(key, document)?;
                    match doc.active {
                        None => return Ok(()),
                        Some(active) if active == key => {}
                        Some(_) => return Err(DomError::InvalidViewTransitionState),
                    }
                    let value = next.get(key)?;
                    // Actual selected participants own these element writes.
                    for record in value.records() {
                        self.source_step(cancelled)?;
                        if let Some(node) = record.branches.new_element() {
                            let key = match self.resolve(node.into()) {
                                Ok(key) => key,
                                Err(DomError::StaleNode) => continue,
                                Err(error) => return Err(error),
                            };
                            self.with_element_fields(key, cancelled, |element| {
                                element.set_captured_in_view_transition(false)
                            })?;
                        }
                    }
                    doc.active = None;
                    doc.value.show = false;
                    // Recreate the admission closure after the element writes.
                    next.set_document(doc, &registry, &mut |usage| {
                        self.admit_custody_usage(usage, cancelled)
                    })?;
                }
                Edit::Release(transition) => {
                    let key = next.transition(&self.owner, self.store, transition.clone())?;
                    next.checked_document(key, document)?;
                    if doc.active == Some(key) {
                        return Err(DomError::InvalidViewTransitionState);
                    }
                    let value = next.get(key)?;
                    next.counts.0 -= 1;
                    next.counts.1 -= value.record_count();
                    next.counts.2 -= value.pseudos;
                    units -= value.name_units;
                    next.remove(key, &registry, &mut admit_storage)?;
                }
            }
        }
        let prepared = self.prepare_undo(cancelled)?;
        let undo = TransitionUndo::capture(self);
        self.view_transitions = next;
        self.units = units;
        let (transitions, records, pseudos) = self.view_transitions.counts;
        self.counts.transitions = transitions;
        self.counts.records = records;
        self.counts.pseudos = pseudos;
        if prepared {
            self.record_transition_undo_prepared(undo);
        }
        Ok(())
    }
    fn admit_transition_name_read(
        &self,
        name: &surgeist_css::values::view_transition::CssViewTransitionIdent,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
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
        if value.document() != Key::Existing(document.node()) {
            return Err(DomError::WrongDocument);
        }
        if !matches!(value.kind(), transaction::KindRead::Element(_)) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(key)
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
#[cfg(test)]
impl ViewTransitions {
    pub(super) fn changes(
        &self,
        old: &Self,
        changes: &mut Vec<DomChange>,
        budget: Option<&source::formation::Formation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut documents = Vec::new();
        source::formation::reserve(
            &mut documents,
            self.documents.record_count(),
            budget,
            cancel,
        )?;
        documents.extend(self.documents.values());
        documents.sort_unstable_by_key(|document| document.init_ordinal);
        for d in documents {
            if old.documents.values().find(|o| o.document == d.document) != Some(d) {
                source::formation::reserve(changes, 1, budget, cancel)?;
                changes.push(DomChange::ViewTransitionDocument(d.document));
            }
        }
        for t in old.transitions.values().chain(&self.transitions) {
            let before = old.transitions.values().find(|o| o.id == t.id);
            let after = self.transitions.values().find(|n| n.id == t.id);
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
                let old_record = before.and_then(|t| t.records.values().find(|old| old.id == r.id));
                let new_record = after.and_then(|t| t.records.values().find(|new| new.id == r.id));
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
                    if let Some(old) = before.records.values().find(|old| old.id == r.id) {
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
    use surgeist_css::values::common::CssCustomIdent;
    use surgeist_css::values::view_transition::CssViewTransitionIdent;
    use surgeist_cssom::{
        CssomContext, CssomEditGuard, CssomInputRole, CssomInputVersion, CssomLimits,
        CssomSheetInputs, CssomStore,
    };

    fn fixture() -> (Dom, DocumentId) {
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                256,
                32,
                0,
                0,
                8,
                8192,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
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
            DomLimits::new(
                8,
                8192,
                32,
                0,
                0,
                8,
                8192,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
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
        crate::DomSourceLimits::new(bytes, 1_000_000, 10_000_000, 64, 4096, 4096, 1)
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

        let (dom, document, transition) = source_append_fixture();
        let before = dom.snapshot();
        let usage = dom.usage();
        let (batch, _, original) = long_append_batch(&dom, document, transition);
        // Administrative radix paths depend on actual materialization IDs.
        // A different owner's observed total is not an exact cost oracle.
        // Derive this occurrence's one-short bound from its actual Draft and
        // the incoming Ident's independently exposed retained capacity.
        let (mut draft, _) = Draft::new_source(
            &dom,
            batch.owner.clone(),
            &batch.commands,
            source_append_limits(4_000_000),
            &cancel,
        )
        .unwrap();
        let existing = draft.source_storage_bytes(&cancel).unwrap() + draft.source_other_bytes;
        let incoming = size_of::<CssViewTransitionIdent>() + original.retained_buffer_bytes();
        draft.source_limits = Some(source_append_limits(existing + incoming - 1));
        let Command::ViewTransition { basis, edit } = &batch.commands[0] else {
            unreachable!()
        };
        append_observation::start();
        let refused = draft.apply_view_transition(*basis, edit, dom.limits(), &cancel);
        let reached = append_observation::finish();
        drop(draft);
        drop(batch);
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
            let (mut draft, _) = Draft::new_source(
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
            assert!(
                draft
                    .view_transitions
                    .get(TransitionKey::Existing(transition))
                    .unwrap()
                    .record_count()
                    == 0
            );
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
        let (mut draft, _) = Draft::new_source(
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
        assert!(
            draft
                .view_transitions
                .get(TransitionKey::Existing(transition))
                .unwrap()
                .record_count()
                == 0
        );
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
            let (mut draft, _) = Draft::new_source(
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
                    4_000_000, 1_000_000, limit, 64, 4096, 4096, 1,
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
            assert_eq!(
                draft
                    .view_transitions
                    .get(TransitionKey::Existing(transition))
                    .unwrap()
                    .record_count(),
                1
            );
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
        storage::set_next_record_for_test(&mut dom, t, u64::MAX);
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
    include!("view_transition/characterization.rs");
    mod discovery_tests {
        use super::*;
        include!("view_transition/discovery_tests.rs");
    }
}
