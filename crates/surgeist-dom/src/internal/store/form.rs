//! Identity-bearing Form map transitions in the same canonical draft.
use super::*;
use crate::internal::{
    element::DomElementDefinition,
    form::{FormNamedScope, FormPastName},
    names::{in_html_namespace_with, same_code_units},
};
use surgeist_html::{attributes::HtmlInputType, vocabulary::KnownHtmlAttribute};

enum FormNameMatches {
    None,
    One(Key),
    Multiple,
}

impl Draft {
    pub(super) fn set_radio_node_list_value(
        &mut self,
        form: Key,
        scope: FormNamedScope,
        name: &DocumentString,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let form = self.require_native_form(form, cancel)?;
        if scope == FormNamedScope::LegacyImages {
            return Ok(());
        }
        let root = crate::internal::slot::root(self, form, cancel)?;
        let mut candidate = Some(root);
        while let Some(key) = candidate {
            self.source_step(cancel)?;
            let node = self.node(key)?;
            if node.control_owner().map(|owner| self.live_key(owner)) == Some(form)
                && node
                    .element()
                    .and_then(|element| element.language().input_state())
                    .is_some_and(|input| input.input_type() == HtmlInputType::Radio)
                && self.form_named_attributes_match(key, name, cancel)?
            {
                let raw = self.control_attribute(key, KnownHtmlAttribute::Value, cancel)?;
                let current = HtmlInputType::Radio
                    .value_mode()
                    .readback(&[], raw.map(DocumentString::code_units), None)
                    .ok_or(DomError::InputInapplicable)?;
                if same_code_units(current, value.code_units(), || self.source_step(cancel))? {
                    return self.select_radio_from_node_list(key, cancel);
                }
            }
            candidate = crate::internal::slot::next(self, root, key, cancel)?;
        }
        Ok(())
    }

    pub(super) fn lookup_form_named(
        &mut self,
        form: Key,
        name: &DocumentString,
        max_retained_query_units: usize,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let form = self.require_native_form(form, cancel)?;
        if !name.is_empty() {
            let root = crate::internal::slot::root(self, form, cancel)?;
            for scope in [FormNamedScope::Controls, FormNamedScope::LegacyImages] {
                match self.find_named_form_members(form, root, scope, name, cancel)? {
                    FormNameMatches::None => {}
                    FormNameMatches::One(target) => {
                        self.register_node_ticket(
                            NodeTicket::FormNamed(target, FormNamedBranch::Element),
                            cancel,
                        )?;
                        return self.record_form_alias(form, name, target, limits, cancel);
                    }
                    FormNameMatches::Multiple => {
                        admit(
                            name.retained_units(),
                            max_retained_query_units,
                            DomResource::HtmlCollectionQueryUnits,
                        )?;
                        return self.register_node_ticket(
                            NodeTicket::FormNamed(form, FormNamedBranch::List(scope)),
                            cancel,
                        );
                    }
                }
            }
            if let Some((_, target, _)) = self.find_form_alias(form, name, cancel)? {
                self.node(target)?;
                return self.register_node_ticket(
                    NodeTicket::FormNamed(target, FormNamedBranch::Element),
                    cancel,
                );
            }
        }
        self.register_node_ticket(NodeTicket::FormNamed(form, FormNamedBranch::Absent), cancel)
    }

    fn require_native_form(&self, form: Key, cancel: &AtomicBool) -> Result<Key, DomError> {
        self.source_step(cancel)?;
        let form = self.live_key(form);
        if !self
            .node(form)?
            .element()
            .is_some_and(|element| element.form_state().is_some())
        {
            return Err(DomError::WrongNodeKind);
        }
        Ok(form)
    }

    fn find_named_form_members(
        &self,
        form: Key,
        root: Key,
        scope: FormNamedScope,
        name: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<FormNameMatches, DomError> {
        let mut first = None;
        let mut candidate = Some(root);
        while let Some(key) = candidate {
            self.source_step(cancel)?;
            let node = self.node(key)?;
            let owner = node.control_owner().map(|owner| self.live_key(owner));
            if owner.is_none_or(|owner| owner == form)
                && node.element().is_some()
                && self.form_named_attributes_match(key, name, cancel)?
                && self.named_form_member(node, owner == Some(form), scope, cancel)?
            {
                if first.is_some() {
                    return Ok(FormNameMatches::Multiple);
                }
                first = Some(key);
            }
            candidate = crate::internal::slot::next(self, root, key, cancel)?;
        }
        Ok(first.map_or(FormNameMatches::None, FormNameMatches::One))
    }

    fn named_form_member(
        &self,
        node: transaction::NodeRead<'_>,
        owner_matches: bool,
        scope: FormNamedScope,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let Some(element) = node.element() else {
            return Ok(false);
        };
        let element = element.language();
        if scope == FormNamedScope::LegacyImages {
            return Ok(owner_matches && node.image().is_some());
        }
        if element.native_listed_form_association().is_some() {
            return Ok(owner_matches
                && !element
                    .input_state()
                    .is_some_and(|input| input.input_type() == HtmlInputType::Image));
        }
        if element.form_associated_custom_disabled().is_some() {
            return Ok(owner_matches);
        }
        if let DomElementDefinition::Definition(definition) = element.custom_element_definition()
            && in_html_namespace_with(element.name(), || self.source_step(cancel))?
        {
            let facts = self
                .creation_registry(definition.registry(), cancel)?
                .definition(definition.ordinal())
                .map_err(DomError::SelectCapture)?;
            if element
                .matches_autonomous_form_definition_with(facts, || self.source_step(cancel))?
            {
                return Err(DomError::IncompleteSourceOperation);
            }
        }
        Ok(false)
    }

    fn form_named_attributes_match(
        &self,
        key: Key,
        name: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        for attribute in [KnownHtmlAttribute::Id, KnownHtmlAttribute::Name] {
            if let Some(candidate) = self.control_attribute(key, attribute, cancel)?
                && same_code_units(candidate.code_units(), name.code_units(), || {
                    self.source_step(cancel)
                })?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn find_form_alias(
        &self,
        form: Key,
        name: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<Option<(usize, Key, usize)>, DomError> {
        let history = self
            .node(form)?
            .form_past_names()
            .ok_or(DomError::WrongNodeKind)?;
        for (position, entry) in history.iter().enumerate() {
            self.source_step(cancel)?;
            if same_code_units(entry.name.code_units(), name.code_units(), || {
                self.source_step(cancel)
            })? {
                return Ok(Some((
                    position,
                    self.live_key(entry.target),
                    entry.name.len(),
                )));
            }
        }
        Ok(None)
    }

    fn record_form_alias(
        &mut self,
        form: Key,
        name: &DocumentString,
        target: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let source = self
            .node(form)?
            .form_past_names()
            .ok_or(DomError::WrongNodeKind)?;
        let previous = self.find_form_alias(form, name, cancel)?;
        if previous.is_some_and(|(position, old, _)| position + 1 == source.len() && old == target)
        {
            return Ok(());
        }
        let next_units = self
            .units
            .checked_sub(previous.map_or(0, |(_, _, units)| units))
            .and_then(|units| units.checked_add(name.len()))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(next_units, limits.units, DomResource::StringCodeUnits)?;
        let count = source
            .len()
            .checked_add(usize::from(previous.is_none()))
            .ok_or(DomError::RepresentationExhausted)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut entries = Vec::new();
        source::formation::reserve(&mut entries, count, formation.as_ref(), cancel)?;
        if let Some(budget) = &formation {
            budget.retain_bytes(size_of::<Vec<FormPastName<Key>>>(), cancel)?;
        }
        for (position, entry) in source.iter().enumerate() {
            self.source_step(cancel)?;
            if previous.is_none_or(|(removed, _, _)| position != removed) {
                entries.push(FormPastName {
                    name: source::formation::share(entry.name, formation.as_ref(), cancel)?,
                    target: entry.target,
                });
            }
        }
        entries.push(FormPastName {
            name: source::formation::share(name, formation.as_ref(), cancel)?,
            target,
        });
        let names = transaction::Names::staged(entries, formation, self, cancel)?;
        {
            let mut fields = self.form_names_mut(form, cancel)?;
            *fields = Some(names);
            fields.commit(cancel)?;
        }
        self.units = next_units;
        Ok(())
    }

    /// HTML877d33343: remove every alias for this exact former control. The
    /// map is read at the intermediate owner state, before any later lookup.
    pub(super) fn remove_form_aliases(
        &mut self,
        owner: Key,
        target: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let source = self
            .node(owner)?
            .form_past_names()
            .expect("actual former Form");
        let mut removed_units = 0usize;
        let mut retained = 0usize;
        for entry in source.iter() {
            self.source_step(cancel)?;
            if entry.target == target {
                removed_units = removed_units
                    .checked_add(entry.name.len())
                    .ok_or(DomError::RepresentationExhausted)?;
            } else {
                retained += 1;
            }
        }
        if retained == source.len() {
            return Ok(());
        }
        // Alias removal never grows the retained map.
        #[cfg(test)]
        let mut entry_writes = crate::internal::store::CopyObservation::default();
        let mut entries = Vec::new();
        let formation = source::formation::Formation::new(self, cancel)?;
        source::formation::reserve(&mut entries, retained, formation.as_ref(), cancel)?;
        if let Some(budget) = &formation
            && entries.capacity() != 0
        {
            budget.retain_bytes(size_of::<Vec<FormPastName<Key>>>(), cancel)?;
        }
        for entry in source.iter() {
            self.source_step(cancel)?;
            if entry.target != target {
                if let Some(budget) = &formation {
                    let mut payload = source::storage::Payload::default();
                    payload.string(entry.name)?;
                    budget.retain_payload(payload, cancel)?;
                }
                entries.push(FormPastName {
                    name: entry.name.clone(),
                    target: entry.target,
                });
                #[cfg(test)]
                entry_writes.pushed::<FormPastName<Key>>();
            }
        }
        let names = transaction::Names::staged(entries, formation, self, cancel)?;
        {
            let mut fields = self.form_names_mut(owner, cancel)?;
            (*fields) = Some(names);
            fields.commit(cancel)
        }?;
        self.units -= removed_units;
        Ok(())
    }
    pub(super) fn retire_form_relations(
        &mut self,
        retired: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        // Explicit retirement differs from ordinary detach/adopt. References
        // cannot survive into a reused slot/generation, even outside the tree.
        let affected = self.inbound_keys(retired, cancel)?;
        for key in affected.iter().copied() {
            self.source_step(cancel)?;
            let Ok(node) = self.node(key) else { continue };
            let clear_owner = node.control_owner() == Some(retired);
            let face = node.element().is_some_and(|element| {
                element
                    .language()
                    .form_associated_custom_disabled()
                    .is_some()
            });
            let form = node.form_past_names().is_some();
            if clear_owner {
                {
                    let mut fields = self.control_fields_mut(key, cancel)?;
                    fields.control_owner = None;
                    fields.commit(cancel)
                }?;
                if face {
                    self.record_form_associated_owner_change(key, Some(retired), None, cancel)?;
                } else {
                    self.input_radio_normalize(
                        key,
                        surgeist_html::elements::input::HtmlRadioTrigger::FormOwner,
                        cancel,
                    )?;
                }
            }
            if form {
                self.remove_form_aliases(key, retired, cancel)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod custody_tests;

    #[test]
    fn alias_pruning_admits_each_examined_entry_even_without_a_match() {
        use crate::internal::source::DomSourceLimits;

        let (mut dom, document, form, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let mut batch = DomBatch::new();
        let unrelated =
            batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "button");
        batch.insert_before(&unrelated, form, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let unrelated = commit.created(&unrelated).unwrap().unwrap();
        for work in [2, 3] {
            let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
            draft.source_limits = Some(DomSourceLimits::new(
                16_000_000, 1_000_000, work, 8, 8, 16, 1,
            ));
            let result =
                draft.remove_form_aliases(Key::Existing(form), Key::Existing(unrelated), &cancel);
            if work == 2 {
                assert_eq!(result, Err(DomError::SourceWorkLimit { limit: 2 }));
            } else {
                assert_eq!(result, Ok(()));
            }
            assert_eq!(draft.source_work.load(Ordering::Relaxed), work);
            assert_eq!(
                draft
                    .node(Key::Existing(form))
                    .unwrap()
                    .form_past_names()
                    .unwrap()
                    .len(),
                3
            );
        }
    }

    fn fixture() -> (Dom, DocumentId, NodeId, NodeId, NodeId) {
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                4096,
                64,
                32,
                16,
                8,
                65536,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let form = b.create_element_ns(&d, surgeist_html::values::Namespace::html(), "form");
        let button = b.create_element_ns(&d, surgeist_html::values::Namespace::html(), "button");
        let field = b.create_element_ns(&d, surgeist_html::values::Namespace::html(), "fieldset");
        b.set_attribute(&form, "id", "f");
        b.insert_before(&form, &d, None);
        b.insert_before(&button, &form, None);
        b.insert_before(&field, &form, None);
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        let d = c.document(&d).unwrap();
        let form = c.created(&form).unwrap().unwrap();
        let button = c.created(&button).unwrap().unwrap();
        let field = c.created(&field).unwrap().unwrap();
        // A finite well-formed retained-history fixture, not public history
        // installation or named lookup.
        let names = vec![
            FormPastName {
                name: "first".into(),
                target: Key::Existing(button),
            },
            FormPastName {
                name: DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
                target: Key::Existing(button),
            },
            FormPastName {
                name: "keep".into(),
                target: Key::Existing(field),
            },
        ];
        // Install private retained-history input through the actual field
        // barrier and publication fence. Raw State mutation would leave its
        // inbound occurrence index and paired custody describing the old map.
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let names = transaction::Names::staged(names, None, &draft, &cancel).unwrap();
        {
            let mut fields = draft.form_names_mut(Key::Existing(form), &cancel).unwrap();
            (*fields) = Some(names);
            fields.commit_formed(None, &cancel)
        }
        .unwrap();
        draft.units += 12; // 5 + 3 + 4 exact UTF-16 code units.
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        crate::internal::store::oracle_tests::assert_full_canonical(&dom);
        (dom, d, form, button, field)
    }
    #[test]
    fn clear_then_reassociate_same_form_removes_all_old_aliases_and_exact_units() {
        let (mut dom, _, form, button, field) = fixture();
        let before = dom.snapshot();
        let units = dom.usage().string_code_units();
        let mut b = DomBatch::new();
        b.set_attribute(button, "form", "f");
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        let s = dom.snapshot();
        assert_eq!(
            s.node(button)
                .unwrap()
                .as_button()
                .unwrap()
                .form_owner()
                .unwrap(),
            Some(form)
        );
        let entries: Vec<_> = s
            .node(form)
            .unwrap()
            .as_form()
            .unwrap()
            .past_names()
            .collect();
        assert_eq!(entries, vec![(&DocumentString::from("keep"), field)]);
        assert_eq!(dom.usage().string_code_units(), units - 8 + 5); // removed names8; new attribute form4 + value1.
        assert!(
            c.changes()
                .changes()
                .contains(&DomChange::ElementState(form))
        );
        assert!(
            !c.changes()
                .changes()
                .contains(&DomChange::ElementState(button))
        ); // final owner unchanged.
        assert_eq!(
            before
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .contains(&DomChange::ElementState(form))
        );
    }
    #[test]
    fn nearest_retention_and_name_change_preserve_aliases_but_native_clone_is_fresh() {
        let (mut dom, d, form, button, _) = fixture();
        let mut b = DomBatch::new();
        b.set_attribute(button, "name", "renamed");
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        assert!(
            !c.changes()
                .changes()
                .contains(&DomChange::ElementState(form))
        );
        let mut b = DomBatch::new();
        b.detach(form);
        b.insert_before(form, d, None);
        dom.apply(b, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            dom.snapshot()
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        let source = dom.snapshot();
        let mut b = DomBatch::new();
        let copy = b.import_node(&source, form, d, true);
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        let copy = c.created(&copy).unwrap().unwrap();
        let s = dom.snapshot();
        assert_eq!(
            s.node(copy).unwrap().as_form().unwrap().past_names().len(),
            0
        );
        assert_eq!(
            s.node(form).unwrap().as_form().unwrap().past_names().len(),
            3
        );
    }
    #[test]
    fn whole_adoption_preserves_history_and_split_clears_only_control_aliases() {
        let (mut dom, _, form, button, field) = fixture();
        let mut b = DomBatch::new();
        let target = b.create_document();
        let c = dom.apply(b, &AtomicBool::new(false)).unwrap();
        let target = c.document(&target).unwrap();
        let mut b = DomBatch::new();
        b.adopt(form, target);
        dom.apply(b, &AtomicBool::new(false)).unwrap();
        assert_eq!(
            dom.snapshot()
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        let mut b = DomBatch::new();
        b.adopt(button, target);
        dom.apply(b, &AtomicBool::new(false)).unwrap();
        let s = dom.snapshot();
        assert_eq!(
            s.node(button)
                .unwrap()
                .as_button()
                .unwrap()
                .form_owner()
                .unwrap(),
            None
        );
        assert_eq!(
            s.node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .collect::<Vec<_>>(),
            vec![(&DocumentString::from("keep"), field)]
        );
    }
    #[test]
    fn explicit_retirement_clears_exact_owner_and_alias_relations() {
        let (dom, _, form, button, field) = fixture();
        let cancel = AtomicBool::new(false);
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        // Exercise the explicit-retirement boundary itself, independently of
        // ordinary removing steps which can already clear a source relation.
        draft
            .retire_form_relations(Key::Existing(button), &cancel)
            .unwrap();
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            1
        );
        draft
            .retire_form_relations(Key::Existing(form), &cancel)
            .unwrap();
        assert_eq!(
            draft.node(Key::Existing(field)).unwrap().control_owner(),
            None
        );
        assert_eq!(
            draft.node(Key::Existing(button)).unwrap().control_owner(),
            None
        );
        assert_eq!(
            dom.snapshot()
                .node(field)
                .unwrap()
                .as_fieldset()
                .unwrap()
                .form_owner()
                .unwrap(),
            Some(form)
        );
    }
    #[test]
    fn prepared_alias_cleanup_cancellation_keeps_original_map_owner_units_and_history() {
        let (mut dom, _, form, button, _) = fixture();
        let before = dom.snapshot();
        let usage = dom.usage();
        let cancel = AtomicBool::new(false);
        let mut b = DomBatch::new();
        b.set_attribute(button, "form", "f");
        let prepared = dom.prepare(b, &cancel).unwrap();
        cancel.store(true, Ordering::Release);
        assert_eq!(
            dom.publish(prepared, &cancel).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.usage(), usage);
        assert_eq!(
            dom.snapshot()
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }

    fn history_identity(draft: &Draft, form: NodeId) -> custody::MaterializationId {
        match draft
            .node(Key::Existing(form))
            .unwrap()
            .form_past_names()
            .unwrap()
        {
            transaction::NamesRead::Empty => panic!("fixture has retained Form history"),
            transaction::NamesRead::Canonical(names)
            | transaction::NamesRead::AliasedCanonical(names, _) => {
                names.payload().unwrap().identity
            }
            transaction::NamesRead::Staged(names)
            | transaction::NamesRead::AliasedStaged(names, _) => names.payload().unwrap().identity,
        }
    }

    #[test]
    fn source_alias_cleanup_restore_refusal_and_replay_keep_original_custody() {
        let (mut dom, _, form, button, field) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let original = dom.state.clone();
        let names = before
            .node(form)
            .unwrap()
            .node
            .form_past_names()
            .unwrap()
            .clone();
        let batch = DomBatch::new();
        let policy = crate::internal::source::DomSourceLimits::new(
            16_000_000, 100_000, 1_000_000, 16, 64, 64, 1,
        );
        let mut draft = Draft::new_source(&dom, batch.owner, &batch.commands, policy, &cancel)
            .unwrap()
            .0;
        draft.custody_registry().reconcile_releases();
        let stable = draft.custody_ledger().total();
        let issuer = payload::identity_refusal::Refusal::begin();
        assert_eq!(
            draft
                .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
                .unwrap_err(),
            DomError::RepresentationExhausted
        );
        assert!(issuer.reached());
        drop(issuer);
        draft.custody_registry().reconcile_releases();
        assert_eq!(draft.custody_ledger().total(), stable);
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            3
        );
        let mark = draft
            .source_checkpoint(draft.source_other_bytes, draft.source_other_units, &cancel)
            .unwrap();
        draft
            .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
            .unwrap();
        let first_id = history_identity(&draft, form);
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            1
        );
        let spent = draft.source_work.load(Ordering::Relaxed);
        mark.restore(&mut draft);
        assert_eq!(draft.source_work.load(Ordering::Relaxed), spent);
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            3
        );
        draft.custody_registry().reconcile_releases();
        draft.source_limits.as_mut().unwrap().retained_bytes =
            draft.custody_ledger().total().bytes() + draft.source_other_bytes;
        assert!(matches!(
            draft.remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            })
        ));
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            3
        );
        assert!(Arc::ptr_eq(&dom.state, &original));
        draft.source_limits = Some(policy);
        // Cancellation must also leave the old group.
        cancel.store(true, Ordering::Relaxed);
        assert_eq!(
            draft
                .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
                .unwrap_err(),
            DomError::Cancelled
        );
        cancel.store(false, Ordering::Relaxed);
        draft
            .remove_form_aliases(Key::Existing(form), Key::Existing(button), &cancel)
            .unwrap();
        let second_id = history_identity(&draft, form);
        assert_ne!(
            first_id, second_id,
            "restored provisional identity is never reused"
        );
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        oracle_tests::assert_full_canonical(&dom);
        let current = dom.snapshot();
        let final_names = current.node(form).unwrap().node.form_past_names().unwrap();
        assert_eq!(
            final_names
                .entries()
                .iter()
                .map(|entry| entry.target)
                .collect::<Vec<_>>(),
            [field]
        );
        assert_eq!(names.entries().len(), 3);
        assert!(names.same_storage(before.node(form).unwrap().node.form_past_names().unwrap()));
    }

    #[test]
    fn source_headroom_executes_alias_cleanup_and_restores_before_real_replay() {
        let (mut dom, _, form, button, field) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_attribute(button, "form", "f");
        let policy = crate::internal::source::DomSourceLimits::new(
            32_000_000, 100_000, 1_000_000, 16, 64, 64, 1,
        );
        let mut draft = Draft::new_source(&dom, batch.owner, &batch.commands, policy, &cancel)
            .unwrap()
            .0;
        let failure = payload::identity_refusal::Refusal::begin();
        assert!(matches!(
            draft.admit_source_command_headroom(
                &dom,
                source::headroom::Entry::Command(&batch.commands[0]),
                0,
                &cancel
            ),
            Err(DomError::RepresentationExhausted)
        ));
        assert!(
            failure.reached(),
            "actual Form materialization was reached by preflight"
        );
        drop(failure);
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            3
        );
        let reservation = draft
            .admit_source_command_headroom(
                &dom,
                source::headroom::Entry::Command(&batch.commands[0]),
                0,
                &cancel,
            )
            .unwrap();
        assert!(reservation.remaining_phases >= 1);
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            3,
            "successful actual preflight restores its provisional group"
        );
        let spent = draft.source_work.load(Ordering::Relaxed);
        draft
            .apply(&batch.commands[0], dom.limits, &cancel)
            .unwrap();
        assert!(draft.source_work.load(Ordering::Relaxed) > spent);
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(draft.source_formation_bytes.load(Ordering::Relaxed), 0);
        assert_eq!(draft.source_temporary_units.load(Ordering::Relaxed), 0);
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        oracle_tests::assert_full_canonical(&dom);
        assert_eq!(
            before
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        assert_eq!(
            dom.snapshot()
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .map(|(_, target)| target)
                .collect::<Vec<_>>(),
            [field]
        );
    }

    #[test]
    fn equal_history_materialization_keeps_original_root_backing_and_history() {
        let (mut dom, _, form, _, _) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let original = dom.state.clone();
        let old = before.node(form).unwrap().node.form_past_names().unwrap();
        let entries = old
            .entries()
            .iter()
            .map(|entry| FormPastName {
                name: entry.name.clone(),
                target: Key::Existing(entry.target),
            })
            .collect();
        let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
        let candidate = transaction::Names::staged(entries, None, &draft, &cancel).unwrap();
        assert_ne!(
            candidate.identity(),
            old.payload().map(|value| value.identity)
        );
        {
            let mut fields = draft.form_names_mut(Key::Existing(form), &cancel).unwrap();
            (*fields) = Some(candidate);
            fields.commit(&cancel)
        }
        .unwrap();
        let prepared = draft.finish(&dom, &cancel).unwrap();
        let commit = dom.publish(prepared, &cancel).unwrap();
        assert!(commit.changes().changes().is_empty());
        assert!(Arc::ptr_eq(&dom.state, &original));
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert!(
            old.same_storage(
                dom.state
                    .node(dom.owner, form)
                    .unwrap()
                    .form_past_names()
                    .unwrap()
            )
        );
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
        oracle_tests::assert_full_canonical(&dom);
    }

    #[test]
    fn public_source_lease_retries_payload_issuer_refusal_without_losing_history() {
        let (mut dom, _, form, button, field) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let policy = crate::internal::source::DomSourceLimits::new(
            32_000_000, 100_000, 1_000_000, 16, 64, 64, 1,
        );
        let mut lease = dom
            .begin_source_edit(
                crate::internal::source::DomSourceEdit::set_native_attribute(button, "form", "f"),
                policy,
                &cancel,
            )
            .unwrap();
        let operation = lease.operation();
        let original_work = lease.status().spent_work();
        let failure = payload::identity_refusal::Refusal::begin();
        assert!(matches!(
            lease.advance(&cancel),
            Err(DomError::RepresentationExhausted)
        ));
        assert!(failure.reached());
        drop(failure);
        let spent = lease.status().spent_work();
        assert!(spent > original_work);
        assert_eq!(lease.operation(), operation);
        assert!(Arc::ptr_eq(&lease.snapshot().state, &before.state));
        assert_eq!(
            lease
                .snapshot()
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        let crate::internal::source::DomSourceProgress::Complete(done) =
            lease.advance(&cancel).unwrap()
        else {
            panic!("no external service is requested by this Form setter")
        };
        assert_eq!(done.operation(), operation);
        drop(lease);
        let current = dom.snapshot();
        assert_eq!(
            current
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .map(|(_, target)| target)
                .collect::<Vec<_>>(),
            [field]
        );
        assert_eq!(
            before
                .node(form)
                .unwrap()
                .as_form()
                .unwrap()
                .past_names()
                .len(),
            3
        );
        oracle_tests::assert_full_canonical(&dom);
    }
}
