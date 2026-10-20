//! Output adapts the HTML ordered cursor to the one canonical Draft.
use super::*;
use surgeist_html::{
    HtmlOutputAction as Action, HtmlOutputCursor, HtmlOutputDefaultSource, HtmlOutputOperation,
};

#[derive(Clone)]
pub(super) struct Frame {
    key: Key,
    cursor: HtmlOutputCursor,
    supplied: Option<DocumentString>,
    read: Option<DocumentString>,
    acknowledge: Option<Action>,
}

impl Frame {
    pub(super) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<source::storage::Payload, DomError> {
        let mut payload = source::storage::Payload::default();
        for value in [self.supplied.as_ref(), self.read.as_ref()]
            .into_iter()
            .flatten()
        {
            step()?;
            payload.string(value)?;
        }
        Ok(payload)
    }
}

impl Draft {
    fn output_state(&self, key: Key) -> Result<&crate::DomOutputState, DomError> {
        let transaction::KindRead::Element(element) = &self.node(key)?.kind() else {
            return Err(DomError::OutputInapplicable);
        };
        element
            .language()
            .output_state()
            .ok_or(DomError::OutputInapplicable)
    }
    fn output_override(
        &mut self,
        key: Key,
        value: Option<&DocumentString>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let old = self
            .output_state(key)?
            .default_override
            .as_ref()
            .map_or(0, DocumentString::len);
        let units = (self.units - old)
            .checked_add(value.map_or(0, DocumentString::len))
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let retained = value
            .map(|value| source::formation::shared_string(self, value, cancel))
            .transpose()?;
        let (value, payload) = match retained {
            Some((value, payload)) => (Some(value), payload),
            None => (None, None),
        };
        let mut state = self.output_state_mut(key, cancel)?;
        state.default_override = value;
        state.commit_charged(payload, cancel)?;
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
            self.source_step(cancel)?;
            let node = self.node(key)?;
            if let transaction::KindRead::Text(text) = &node.kind() {
                visit(text)?;
            }
            if let Some(child) = node.children().first() {
                key = child;
                continue;
            }
            loop {
                self.source_step(cancel)?;
                if key == root {
                    return Ok(());
                }
                let parent = self
                    .node(key)?
                    .parent()
                    .expect("ordinary descendant has parent");
                let mut found = false;
                let mut next = None;
                for child in self.node(parent)?.children().iter() {
                    self.source_step(cancel)?;
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
    ) -> Result<(DocumentString, Option<source::checkpoint::Charge>), DomError> {
        let state = self.output_state(key)?;
        if surgeist_html::output_default_source(state.default_override.is_some())
            == HtmlOutputDefaultSource::Override
        {
            return source::formation::shared_string(
                self,
                state.default_override.as_ref().expect("actual override"),
                cancel,
            );
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
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::Output,
            cancel,
        )?;
        let formation = source::formation::Formation::new(self, cancel)?;
        if let Some(formation) = &formation {
            formation.retain_payload(
                source::storage::Payload {
                    bytes: size_of::<Vec<u16>>(),
                    units: length,
                },
                cancel,
            )?;
        }
        let mut output = Vec::new();
        if formation.is_some() {
            source::formation::reserve(&mut output, length, formation.as_ref(), cancel)?;
        } else {
            output
                .try_reserve_exact(length)
                .map_err(|_| DomError::AllocationFailed)?;
            #[cfg(test)]
            if scale_metrics::ENABLED && output.capacity() != 0 {
                scale_metrics::record(scale_metrics::Event::StructuralAllocation {
                    bytes: output.capacity() * size_of::<u16>(),
                });
            }
        }
        if let Some(formation) = &formation {
            formation.retain_payload(
                source::storage::Payload {
                    bytes: 0,
                    units: output
                        .capacity()
                        .checked_sub(length)
                        .ok_or(DomError::RepresentationExhausted)?,
                },
                cancel,
            )?;
        }
        admit(
            output.capacity(),
            limits.units,
            DomResource::StringCodeUnits,
        )?;
        #[cfg(test)]
        let mut written = transaction::CopyObservation::default();
        self.output_text(key, cancel, |text| {
            for unit in text.code_units() {
                self.source_step(cancel)?;
                output.push(*unit);
                #[cfg(test)]
                written.pushed::<u16>();
            }
            Ok(())
        })?;
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            size_of::<Vec<u16>>() + output.capacity() * size_of::<u16>(),
        );
        Ok((
            DocumentString::from_utf16(output),
            formation.map(source::formation::Formation::into_charge),
        ))
    }
    pub(super) fn output_edit(
        &mut self,
        key: Key,
        edit: &crate::output::Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let (frame, payload) = self.output_frame(key, edit, cancel)?;
        self.push_effect_retained(
            select::Effect::Output(frame),
            self.effect_scoped,
            payload,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    fn output_frame(
        &self,
        key: Key,
        edit: &crate::output::Edit,
        cancel: &AtomicBool,
    ) -> Result<(Frame, Option<source::checkpoint::Charge>), DomError> {
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
        let (supplied, payload) = match supplied {
            Some(value) => {
                let (value, payload) = source::formation::shared_string(self, value, cancel)?;
                (Some(value), payload)
            }
            None => (None, None),
        };
        Ok((
            Frame {
                key,
                cursor: HtmlOutputCursor::new(operation),
                supplied,
                read: None,
                acknowledge: None,
            },
            payload,
        ))
    }
    // Both the canonical frame executor and the cancellation characterization
    // consume this same source action. A replacement returns its operand before
    // acknowledgement; its owning frame survives until all nested hooks finish.
    fn output_action(
        &mut self,
        frame: &mut Frame,
        payload: &mut Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<Option<(DocumentString, Option<source::checkpoint::Charge>)>, DomError> {
        let action = frame.cursor.action();
        match action {
            Action::ReadDefaultValue => {
                let (read, incoming) = self.output_default(frame.key, limits, cancel)?;
                frame.read = Some(read);
                source::formation::merge_charge(payload, incoming);
            }
            Action::StoreReadDefaultAsOverride => self.output_override(
                frame.key,
                Some(frame.read.as_ref().expect("cursor read default")),
                limits,
                cancel,
            )?,
            Action::StoreSuppliedValueAsOverride => self.output_override(
                frame.key,
                Some(frame.supplied.as_ref().expect("cursor supplied value")),
                limits,
                cancel,
            )?,
            Action::ReplaceAllWithSuppliedValue => {
                return source::formation::shared_string(
                    self,
                    frame.supplied.as_ref().expect("cursor supplied value"),
                    cancel,
                )
                .map(Some);
            }
            Action::ReplaceAllWithReadDefault => {
                return source::formation::shared_string(
                    self,
                    frame.read.as_ref().expect("cursor read default"),
                    cancel,
                )
                .map(Some);
            }
            Action::ClearDefaultOverride => {
                self.output_override(frame.key, None, limits, cancel)?
            }
            Action::Complete => unreachable!("complete cursor is not an action"),
        }
        frame
            .cursor
            .acknowledge(action)
            .map_err(DomError::OutputProtocol)?;
        Ok(None)
    }
    pub(super) fn output_effect(
        &mut self,
        mut frame: Frame,
        mut payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if let Some(action) = frame.acknowledge.take() {
            frame
                .cursor
                .acknowledge(action)
                .map_err(DomError::OutputProtocol)?;
        }
        if frame.cursor.action() == Action::Complete {
            return Ok(());
        }
        check_cancel(cancel)?;
        if let Some((value, replacement_payload)) =
            self.output_action(&mut frame, &mut payload, limits, cancel)?
        {
            frame.acknowledge = Some(frame.cursor.action());
            let key = frame.key;
            self.push_effect_retained(
                select::Effect::Output(frame),
                self.effect_scoped,
                payload,
                cancel,
            )?;
            self.string_replace_all(key, value, replacement_payload, limits, cancel)
        } else {
            self.push_effect_retained(
                select::Effect::Output(frame),
                self.effect_scoped,
                payload,
                cancel,
            )
        }
    }
    #[cfg(test)]
    fn output_edit_admitted(
        &mut self,
        key: Key,
        edit: crate::output::Edit,
        limits: DomLimits,
        cancel: &AtomicBool,
        admit_action: &mut impl FnMut(Action) -> Result<(), DomError>,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let (mut frame, mut payload) = self.output_frame(key, &edit, cancel)?;
        while frame.cursor.action() != Action::Complete {
            let action = frame.cursor.action();
            admit_action(action)?;
            if let Some((value, replacement_payload)) =
                self.output_action(&mut frame, &mut payload, limits, cancel)?
            {
                self.string_replace_all(key, value, replacement_payload, limits, cancel)?;
                frame
                    .cursor
                    .acknowledge(action)
                    .map_err(DomError::OutputProtocol)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_text_reports_the_actual_initialized_utf16_output() {
        use crate::store::scale_metrics::{Hooks, Measurement};
        if !scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                4096,
                32,
                0,
                0,
                8,
                65_536,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let output = batch.create_element_ns(&document, crate::Namespace::html(), "output");
        let first = batch.create_text(&document, "ab");
        let second = batch.create_text(&document, "cd");
        batch.insert_before(&first, &output, None);
        batch.insert_before(&second, &output, None);
        let commit = dom.apply(batch, &cancel).unwrap();
        let output = commit.created(&output).unwrap().unwrap();
        for source in [false, true] {
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
            let measurement = Measurement::begin().unwrap();
            let output = draft
                .output_default(Key::Existing(output), dom.limits, &cancel)
                .unwrap();
            let counters = measurement.finish(Hooks::default()).unwrap();
            assert_eq!(output.0, DocumentString::from("abcd"));
            assert_eq!(counters.sequence_materialized_bytes, 4 * size_of::<u16>());
            assert_eq!(counters.relation_sequence_materialized_bytes, 0);
            assert!(counters.structural_bytes >= 4 * size_of::<u16>());
        }
    }

    #[test]
    fn source_prefix_store_and_reset_replace_reach_cancellation_without_publishing() {
        for reset in [false, true] {
            let mut dom = Dom::new(
                DomLimits::new(
                    32,
                    4096,
                    64,
                    0,
                    0,
                    8,
                    65536,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
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
            let old_children = draft
                .node(key)
                .unwrap()
                .children()
                .iter()
                .collect::<Vec<_>>();
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
                assert_eq!(draft.node(input_key).unwrap().control_owner(), None);
                assert_eq!(draft.node(input_key).unwrap().parent(), None);
                assert_eq!(draft.node(key).unwrap().children().len(), 1);
                let child = draft.node(key).unwrap().children().get(0).unwrap();
                assert!(
                    matches!(draft.node(child).unwrap().kind(),transaction::KindRead::Text(v) if v.code_units() == [111,108,100])
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
                assert_eq!(
                    draft
                        .node(key)
                        .unwrap()
                        .children()
                        .iter()
                        .collect::<Vec<_>>(),
                    old_children
                );
                assert_eq!(
                    draft.node(input_key).unwrap().control_owner(),
                    Some(form_key)
                );
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
            let mut dom = Dom::new(
                DomLimits::new(
                    32,
                    4096,
                    64,
                    0,
                    0,
                    8,
                    65536,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
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
            let DomNodeKind::Element(element) = &actual.kind() else {
                unreachable!()
            };
            assert_eq!(
                element.output_state().unwrap().default_value_override(),
                Some(&DocumentString::from("new default"))
            );
            assert_eq!(actual.control_owner(), None);
            assert_eq!(actual.parent(), None);
            assert_eq!(actual.children().len(), 1);
            assert_eq!(
                actual.document().node(),
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
