//! Identity-bearing Form map transitions in the same canonical draft.
use super::*;
use crate::form::{FormPastName, FormPastNames};

pub(super) fn map_names_formed<A: Copy, B>(
    source: &FormPastNames<A>,
    mut map: impl FnMut(A) -> B,
    budget: Option<&style::formation::Formation>,
    cancel: &AtomicBool,
) -> Result<FormPastNames<B>, DomError> {
    let mut entries = Vec::new();
    style::formation::reserve(&mut entries, source.entries.len(), budget, cancel)?;
    for entry in source.entries.iter() {
        check_cancel(cancel)?;
        entries.push(FormPastName {
            name: entry.name.clone(),
            target: map(entry.target),
        });
    }
    Ok(FormPastNames {
        entries: Arc::new(entries),
    })
}
pub(super) fn name_units<K>(
    source: Option<&FormPastNames<K>>,
    cancel: &AtomicBool,
) -> Result<usize, DomError> {
    let mut units = 0usize;
    if let Some(source) = source {
        for entry in source.entries.iter() {
            check_cancel(cancel)?;
            units = units
                .checked_add(entry.name.len())
                .ok_or(DomError::RepresentationExhausted)?;
        }
    }
    Ok(units)
}
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
            .as_ref()
            .expect("actual former Form");
        let mut removed_units = 0usize;
        let mut retained = 0usize;
        for entry in source.entries.iter() {
            check_cancel(cancel)?;
            if entry.target == target {
                removed_units = removed_units
                    .checked_add(entry.name.len())
                    .ok_or(DomError::RepresentationExhausted)?;
            } else {
                retained += 1;
            }
        }
        if retained == source.entries.len() {
            return Ok(());
        }
        // Current public fronts cannot grow maps. This replacement cannot
        // exceed the actual incoming retained extent; there is no unlimited
        // admission fallback or unrelated Slot-reference quota.
        let mut entries = Vec::new();
        reserve(&mut entries, retained)?;
        for entry in source.entries.iter() {
            check_cancel(cancel)?;
            if entry.target != target {
                entries.push(entry.clone());
            }
        }
        self.node_mut(owner).form_past_names = Some(FormPastNames {
            entries: Arc::new(entries),
        });
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
        for index in 0..self.existing.len() + self.created.len() {
            check_cancel(cancel)?;
            let key = if index < self.existing.len() {
                Key::Existing(index)
            } else {
                Key::Created(index - self.existing.len())
            };
            let Ok(node) = self.node(key) else { continue };
            let clear_owner = node.control_owner == Some(retired);
            let form = node.form_past_names.is_some();
            if clear_owner {
                self.node_mut(key).control_owner = None;
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
    fn fixture() -> (Dom, DocumentId, NodeId, NodeId, NodeId) {
        let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 32, 16, 8, 65536).unwrap()).unwrap();
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
                target: button,
            },
            FormPastName {
                name: DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
                target: button,
            },
            FormPastName {
                name: "keep".into(),
                target: field,
            },
        ];
        let state = Arc::get_mut(&mut dom.state).unwrap();
        state.slots[form.slot]
            .node
            .as_mut()
            .unwrap()
            .form_past_names = Some(FormPastNames {
            entries: Arc::new(names),
        });
        state.units += 12; // 5 + 3 + 4 exact UTF-16 code units.
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
            .retire_form_relations(Key::Existing(button.slot), &cancel)
            .unwrap();
        assert_eq!(
            draft
                .node(Key::Existing(form.slot))
                .unwrap()
                .form_past_names
                .as_ref()
                .unwrap()
                .entries
                .len(),
            1
        );
        draft
            .retire_form_relations(Key::Existing(form.slot), &cancel)
            .unwrap();
        assert_eq!(
            draft.node(Key::Existing(field.slot)).unwrap().control_owner,
            None
        );
        assert_eq!(
            draft
                .node(Key::Existing(button.slot))
                .unwrap()
                .control_owner,
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
}
