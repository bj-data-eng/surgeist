//! Identity-bearing Form map transitions in the same canonical draft.
use super::*;
use crate::form::FormPastName;

impl Draft {
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
            .form_past_names
            .expect("actual former Form");
        let mut removed_units = 0usize;
        let mut retained = 0usize;
        for entry in source.iter() {
            check_cancel(cancel)?;
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
        // Current public fronts cannot grow maps. This replacement cannot
        // exceed the actual incoming retained extent; there is no unlimited
        // admission fallback or unrelated Slot-reference quota.
        #[cfg(test)]
        let mut entry_writes = crate::store::CopyObservation::default();
        let mut entries = Vec::new();
        let formation = source::formation::Formation::new(self, cancel)?;
        source::formation::reserve(&mut entries, retained, formation.as_ref(), cancel)?;
        if let Some(budget) = &formation
            && entries.capacity() != 0
        {
            budget.retain_bytes(size_of::<Vec<FormPastName<Key>>>(), cancel)?;
        }
        for entry in source.iter() {
            check_cancel(cancel)?;
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
        self.with_node_fields(owner, cancel, |fields| fields.form_past_names = Some(names))?;
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
            let clear_owner = node.control_owner == Some(retired);
            let form = node.form_past_names.is_some();
            if clear_owner {
                self.with_node_fields(key, cancel, |fields| fields.control_owner = None)?;
                self.input_radio_normalize(
                    key,
                    surgeist_html::HtmlRadioTrigger::FormOwner,
                    cancel,
                )?;
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
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut b = DomBatch::new();
        let d = b.create_document();
        let form = b.create_element_ns(&d, crate::Namespace::html(), "form");
        let button = b.create_element_ns(&d, crate::Namespace::html(), "button");
        let field = b.create_element_ns(&d, crate::Namespace::html(), "fieldset");
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
        // installation or named lookup. Native APIs admit no entries today.
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
        draft
            .with_node_fields_formed(Key::Existing(form), &cancel, None, |fields| {
                fields.form_past_names = Some(names);
            })
            .unwrap();
        draft.units += 12; // 5 + 3 + 4 exact UTF-16 code units.
        let prepared = draft.finish(&dom, &cancel).unwrap();
        dom.publish(prepared, &cancel).unwrap();
        crate::store::oracle_tests::assert_full_canonical(&dom);
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
                .form_past_names
                .as_ref()
                .unwrap()
                .len(),
            1
        );
        draft
            .retire_form_relations(Key::Existing(form), &cancel)
            .unwrap();
        assert_eq!(
            draft.node(Key::Existing(field)).unwrap().control_owner,
            None
        );
        assert_eq!(
            draft.node(Key::Existing(button)).unwrap().control_owner,
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
            .form_past_names
            .as_ref()
            .unwrap()
            .clone();
        let batch = DomBatch::new();
        let policy = crate::DomSourceLimits::new(16_000_000, 100_000, 1_000_000, 16, 64, 64, 1);
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
                .form_past_names
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
        let first_id = draft
            .records_for(Key::Existing(form))
            .fields
            .value()
            .form_past_names
            .as_ref()
            .unwrap()
            .identity()
            .unwrap();
        assert_eq!(
            draft
                .node(Key::Existing(form))
                .unwrap()
                .form_past_names
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
                .form_past_names
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
                .form_past_names
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
        let second_id = draft
            .records_for(Key::Existing(form))
            .fields
            .value()
            .form_past_names
            .as_ref()
            .unwrap()
            .identity()
            .unwrap();
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
        let final_names = current
            .node(form)
            .unwrap()
            .node
            .form_past_names
            .as_ref()
            .unwrap();
        assert_eq!(
            final_names
                .entries()
                .iter()
                .map(|entry| entry.target)
                .collect::<Vec<_>>(),
            [field]
        );
        assert_eq!(names.entries().len(), 3);
        assert!(
            names.same_storage(
                before
                    .node(form)
                    .unwrap()
                    .node
                    .form_past_names
                    .as_ref()
                    .unwrap()
            )
        );
    }

    #[test]
    fn source_headroom_executes_alias_cleanup_and_restores_before_real_replay() {
        let (mut dom, _, form, button, field) = fixture();
        let cancel = AtomicBool::new(false);
        let before = dom.snapshot();
        let mut batch = DomBatch::new();
        batch.set_attribute(button, "form", "f");
        let policy = crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1);
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
                .form_past_names
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
                .form_past_names
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
                .form_past_names
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
        let old = before
            .node(form)
            .unwrap()
            .node
            .form_past_names
            .as_ref()
            .unwrap();
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
        draft
            .with_node_fields(Key::Existing(form), &cancel, |fields| {
                fields.form_past_names = Some(candidate)
            })
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
                    .form_past_names
                    .as_ref()
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
        let policy = crate::DomSourceLimits::new(32_000_000, 100_000, 1_000_000, 16, 64, 64, 1);
        let mut lease = dom
            .begin_source_edit(
                crate::DomSourceEdit::set_attribute(button, "form", "f"),
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
        let crate::DomSourceProgress::Complete(done) = lease.advance(&cancel).unwrap() else {
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
