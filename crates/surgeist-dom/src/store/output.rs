//! Output adapts the HTML ordered cursor to the one canonical Draft.
use super::*;
use surgeist_html::{
    HtmlOutputAction as Action, HtmlOutputCursor, HtmlOutputDefaultSource, HtmlOutputOperation,
};

impl Draft {
    fn output_state(&self, key: Key) -> Result<&crate::DomOutputState, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Err(DomError::OutputInapplicable);
        };
        element.output_state().ok_or(DomError::OutputInapplicable)
    }
    fn output_override(
        &mut self,
        key: Key,
        value: Option<DocumentString>,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        let old = self
            .output_state(key)?
            .default_override
            .as_ref()
            .map_or(0, DocumentString::len);
        let units = (self.units - old)
            .checked_add(value.as_ref().map_or(0, DocumentString::len))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
            unreachable!("checked Output")
        };
        element
            .output_state_mut()
            .expect("checked Output")
            .default_override = value;
        self.units = units;
        Ok(())
    }
    // The ordinary parent/sibling links provide finite traversal without a second
    // retained tree or a traversal allocation. Association-owned Template and
    // ShadowRoot edges are deliberately not child edges.
    fn output_text(
        &self,
        root: Key,
        cancel: &AtomicBool,
        mut visit: impl FnMut(&DocumentString) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        let mut key = root;
        loop {
            check_cancel(cancel)?;
            let node = self.node(key)?;
            if let DomNodeKind::Text(text) = &node.kind {
                visit(text)?;
            }
            if let Some(&child) = node.children.first() {
                key = child;
                continue;
            }
            loop {
                check_cancel(cancel)?;
                if key == root {
                    return Ok(());
                }
                let parent = self
                    .node(key)?
                    .parent
                    .expect("ordinary descendant has parent");
                let mut found = false;
                let mut next = None;
                for &child in &self.node(parent)?.children {
                    check_cancel(cancel)?;
                    if found {
                        next = Some(child);
                        break;
                    }
                    found = child == key;
                }
                if let Some(next) = next {
                    key = next;
                    break;
                }
                key = parent;
            }
        }
    }
    fn output_default(
        &self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        let state = self.output_state(key)?;
        if surgeist_html::output_default_source(state.default_override.is_some())
            == HtmlOutputDefaultSource::Override
        {
            return Ok(state
                .default_override
                .as_ref()
                .expect("actual override")
                .clone());
        }
        let mut length = 0usize;
        self.output_text(key, cancel, |text| {
            length = length
                .checked_add(text.len())
                .ok_or(DomError::RepresentationExhausted)?;
            admit(length, limits.units, DomResource::StringCodeUnits)
        })?;
        length
            .checked_mul(std::mem::size_of::<u16>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        check_cancel(cancel)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(length)
            .map_err(|_| DomError::AllocationFailed)?;
        admit(
            output.capacity(),
            limits.units,
            DomResource::StringCodeUnits,
        )?;
        self.output_text(key, cancel, |text| {
            for &unit in text.code_units() {
                check_cancel(cancel)?;
                output.push(unit);
            }
            Ok(())
        })?;
        Ok(DocumentString::from_utf16(output))
    }
    pub(super) fn output_edit(
        &mut self,
        key: Key,
        edit: crate::output::Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.output_edit_admitted(key, edit, limits, cancel, &mut |_| check_cancel(cancel))
    }
    fn output_edit_admitted(
        &mut self,
        key: Key,
        edit: crate::output::Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_action: &mut impl FnMut(Action) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let state = self.output_state(key)?;
        let (operation, supplied) = match edit {
            crate::output::Edit::Value(value) => (HtmlOutputOperation::ValueSetter, Some(value)),
            crate::output::Edit::DefaultValue(value) => (
                HtmlOutputOperation::DefaultValueSetter {
                    has_override: state.default_override.is_some(),
                },
                Some(value),
            ),
            crate::output::Edit::Reset => (HtmlOutputOperation::Reset, None),
        };
        let mut cursor = HtmlOutputCursor::new(operation);
        let mut read = None;
        loop {
            let action = cursor.action();
            if action == Action::Complete {
                return Ok(());
            }
            admit_action(action)?;
            match action {
                Action::ReadDefaultValue => read = Some(self.output_default(key, limits, cancel)?),
                Action::StoreReadDefaultAsOverride => self.output_override(
                    key,
                    Some(read.as_ref().expect("cursor read default").clone()),
                    limits,
                )?,
                Action::ReplaceAllWithSuppliedValue => self.string_replace_all(
                    key,
                    supplied.as_ref().expect("cursor supplied value").clone(),
                    limits,
                    cancel,
                )?,
                Action::StoreSuppliedValueAsOverride => self.output_override(
                    key,
                    Some(supplied.as_ref().expect("cursor supplied value").clone()),
                    limits,
                )?,
                Action::ReplaceAllWithReadDefault => self.string_replace_all(
                    key,
                    read.as_ref().expect("cursor read default").clone(),
                    limits,
                    cancel,
                )?,
                Action::ClearDefaultOverride => self.output_override(key, None, limits)?,
                Action::Complete => unreachable!("completed cursor returned"),
            }
            cursor
                .acknowledge(action)
                .map_err(DomError::OutputProtocol)?;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_prefix_store_and_reset_replace_reach_cancellation_without_publishing() {
        for reset in [false, true] {
            let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 65536).unwrap()).unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let d = b.create_document();
            let form = b.create_element_ns(&d, crate::Namespace::html(), "form");
            let output = b.create_element_ns(&d, crate::Namespace::html(), "output");
            let input = b.create_element_ns(&d, crate::Namespace::html(), "input");
            b.insert_before(&form, &d, None);
            b.insert_before(&output, &form, None);
            b.insert_before(&input, &output, None);
            let text = b.create_text(&d, "old");
            b.insert_before(&text, &output, None);
            if reset {
                b.set_output_value(&output, "current");
                // Reinsert the real control so reset's actual removal changes
                // its canonical owner before the final clear is canceled.
                b.insert_before(&input, &output, None);
            }
            let c = dom.apply(b, &cancel).unwrap();
            let output = c.created(&output).unwrap().unwrap();
            let input = c.created(&input).unwrap().unwrap();
            let form = c.created(&form).unwrap().unwrap();
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
            let key = draft.resolve(output.into()).unwrap();
            let input_key = draft.resolve(input.into()).unwrap();
            let form_key = draft.resolve(form.into()).unwrap();
            let old_children = draft.node(key).unwrap().children.clone();
            let mut actions = Vec::new();
            let stop = if reset {
                Action::ClearDefaultOverride
            } else {
                Action::ReplaceAllWithSuppliedValue
            };
            let result = draft.output_edit_admitted(
                key,
                if reset {
                    crate::output::Edit::Reset
                } else {
                    crate::output::Edit::Value("new".into())
                },
                dom.limits,
                &cancel,
                &mut |action| {
                    actions.push(action);
                    if action == stop {
                        cancel.store(true, Ordering::Release);
                    }
                    check_cancel(&cancel)
                },
            );
            assert_eq!(result, Err(DomError::Cancelled));
            assert_eq!(
                draft.output_state(key).unwrap().default_value_override(),
                Some(&DocumentString::from("old"))
            );
            if reset {
                assert_eq!(
                    actions,
                    vec![
                        Action::ReadDefaultValue,
                        Action::ReplaceAllWithReadDefault,
                        Action::ClearDefaultOverride
                    ]
                );
                assert_eq!(draft.node(input_key).unwrap().control_owner, None);
                assert_eq!(draft.node(input_key).unwrap().parent, None);
                assert_eq!(draft.node(key).unwrap().children.len(), 1);
                let child = draft.node(key).unwrap().children[0];
                assert!(
                    matches!(&draft.node(child).unwrap().kind,DomNodeKind::Text(v) if v == &DocumentString::from("old"))
                );
            } else {
                assert_eq!(
                    actions,
                    vec![
                        Action::ReadDefaultValue,
                        Action::StoreReadDefaultAsOverride,
                        Action::ReplaceAllWithSuppliedValue
                    ]
                );
                assert_eq!(draft.node(key).unwrap().children, old_children);
                assert_eq!(draft.node(input_key).unwrap().control_owner, Some(form_key));
            }
            drop(draft);
            assert_eq!(*dom.state, *before.state);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert!(
                dom.changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }

    #[test]
    fn fully_prepared_tree_owner_and_override_changes_refuse_cancel_or_revision_exhaustion() {
        for exhausted in [false, true] {
            let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 65536).unwrap()).unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let d = b.create_document();
            let form = b.create_element_ns(&d, crate::Namespace::html(), "form");
            let output = b.create_element_ns(&d, crate::Namespace::html(), "output");
            b.insert_before(&form, &d, None);
            b.insert_before(&output, &form, None);
            b.set_output_default_value(&output, "default");
            let c = dom.apply(b, &cancel).unwrap();
            let output = c.created(&output).unwrap().unwrap();
            if exhausted {
                dom.revision = crate::DomRevision(u64::MAX);
            }
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            let newdoc = b.create_document();
            b.set_output_value(output, "current");
            b.set_output_default_value(output, "new default");
            b.adopt(output, &newdoc);
            let prepared = dom.prepare(b, &cancel).unwrap();
            let actual = prepared.state.node(dom.owner, output).unwrap();
            let DomNodeKind::Element(element) = &actual.kind else {
                unreachable!()
            };
            assert_eq!(
                element.output_state().unwrap().default_value_override(),
                Some(&DocumentString::from("new default"))
            );
            assert_eq!(actual.control_owner, None);
            assert_eq!(actual.parent, None);
            assert_eq!(actual.children.len(), 1);
            assert_eq!(
                actual.document.node(),
                prepared.created[newdoc.index].unwrap()
            );
            if !exhausted {
                cancel.store(true, Ordering::Release);
            }
            assert_eq!(
                dom.publish(prepared, &cancel).unwrap_err(),
                if exhausted {
                    DomError::RevisionExhausted
                } else {
                    DomError::Cancelled
                }
            );
            assert_eq!(*dom.state, *before.state);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert!(
                dom.changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }
}
