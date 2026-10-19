//! Textarea's source-ordered raw, child, value, reset and clone transitions.
use super::*;

impl Draft {
    pub(super) fn textarea_state(&self, key: Key) -> Result<&crate::DomTextareaState, DomError> {
        let DomNodeKind::Element(element) = self.node(key)?.kind else {
            return Err(DomError::TextareaInapplicable);
        };
        element
            .textarea_state()
            .ok_or(DomError::TextareaInapplicable)
    }
    fn with_textarea_state<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        edit: impl FnOnce(&mut crate::DomTextareaState) -> R,
    ) -> Result<R, DomError> {
        self.prepare_family_mutation(key, cancel)?;
        self.with_element_fields(key, cancel, |element| {
            edit(element.textarea_state_mut().expect("checked Textarea"))
        })
    }
    fn with_textarea_state_charged<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        payload: Option<source::checkpoint::Charge>,
        edit: impl FnOnce(&mut crate::DomTextareaState) -> R,
    ) -> Result<R, DomError> {
        self.prepare_family_mutation(key, cancel)?;
        self.with_element_fields_charged(key, cancel, payload, |element| {
            edit(element.textarea_state_mut().expect("checked Textarea"))
        })
    }
    fn textarea_put_raw(
        &mut self,
        key: Key,
        raw: DocumentString,
        payload: Option<source::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        struct Raw {
            value: DocumentString,
            payload: Option<source::checkpoint::Charge>,
        }
        let raw = Raw {
            value: raw,
            payload,
        };
        let units = (self.units - self.textarea_state(key)?.raw.len())
            .checked_add(raw.value.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let length = crate::textarea::normalized_length(raw.value.code_units(), &mut || {
            self.source_step(cancel)
        })?;
        let platform = self
            .document_metadata(self.node(key)?.document)?
            .document_environment()
            .input_platform();
        let selection = self
            .textarea_state(key)?
            .selection
            .clamp(length, platform.empty_selection());
        let Raw {
            value: raw,
            payload,
        } = raw;
        self.with_textarea_state_charged(key, cancel, payload, |state| {
            state.selection = selection;
            state.raw = raw;
        })?;
        self.units = units;
        Ok(())
    }
    fn textarea_child_text(
        &self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(DocumentString, Option<source::checkpoint::Charge>), DomError> {
        let mut length = 0usize;
        for child in self.node(key)?.children.iter() {
            self.source_step(cancel)?;
            if let DomNodeKind::Text(text) = &self.node(child)?.kind {
                length = length
                    .checked_add(text.len())
                    .ok_or(DomError::RepresentationExhausted)?;
            }
        }
        let units = (self.units - self.textarea_state(key)?.raw.len())
            .checked_add(length)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        #[cfg(test)]
        let mut output_writes = crate::store::CopyObservation::default();
        let mut output = crate::textarea::output(length, limits.units, formation.as_ref(), cancel)?;
        for child in self.node(key)?.children.iter() {
            self.source_step(cancel)?;
            if let DomNodeKind::Text(text) = &self.node(child)?.kind {
                for unit in text.code_units() {
                    self.source_step(cancel)?;
                    output.push(*unit);
                    #[cfg(test)]
                    output_writes.pushed::<u16>();
                }
            }
        }
        Ok((
            DocumentString::from_utf16(output),
            formation.map(source::formation::Formation::into_charge),
        ))
    }
    pub(super) fn textarea_children_changed(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Ok(state) = self.textarea_state(key) else {
            return Ok(());
        };
        if state.flags.children_changed()
            == surgeist_html::HtmlTextareaRawValueAction::ReplaceWithChildText
        {
            let (raw, payload) = self.textarea_child_text(key, limits, cancel)?;
            self.textarea_put_raw(key, raw, payload, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn set_textarea_value(
        &mut self,
        key: Key,
        raw: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let previous = &self.textarea_state(key)?.raw;
        let units = (self.units - previous.len())
            .checked_add(raw.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::TextareaApi,
            cancel,
        )?;
        let old_api_formation = source::formation::Formation::new(self, cancel)?;
        let old_api = crate::textarea::normalized_value(
            previous.code_units(),
            previous.len(),
            &mut || self.source_step(cancel),
            old_api_formation.as_ref(),
            cancel,
        )?;
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            size_of::<Vec<u16>>() + old_api.shared_code_units().capacity() * size_of::<u16>(),
        );
        let (raw, payload) = source::formation::shared_string(self, raw, cancel)?;
        self.textarea_put_raw(key, raw, payload, limits, cancel)?;
        self.with_textarea_state(key, cancel, |state| state.flags.mark_dirty_value())?;
        let current = &self.textarea_state(key)?.raw;
        let new_api_formation = source::formation::Formation::new(self, cancel)?;
        let new_api = crate::textarea::normalized_value(
            current.code_units(),
            current.len(),
            &mut || self.source_step(cancel),
            new_api_formation.as_ref(),
            cancel,
        )?;
        let mut cursor = surgeist_html::HtmlInputValueComparisonCursor::new(
            old_api.code_units(),
            new_api.code_units(),
        );
        while cursor.required_work().is_some() {
            self.source_step(cancel)?;
            cursor.advance();
        }
        if surgeist_html::textarea_value_setter_finish(
            cursor.result().expect("completed actual API comparison"),
        )
            == surgeist_html::HtmlTextControlValueSetterFinish::SetEndCursorUnselectAndNoneDirection
        {
            let platform = self
                .document_metadata(self.node(key)?.document)?
                .document_environment()
                .input_platform();
            self.with_textarea_state(key, cancel, |state| {
                state.selection = surgeist_html::HtmlInputSelection::cursor(
                    new_api.len(),
                    surgeist_html::HtmlInputSelectionDirection::None
                        .resolve_none(platform.none_direction()),
                )
            })?;
        }
        drop(new_api);
        drop(new_api_formation);
        drop(old_api);
        drop(old_api_formation);
        Ok(())
    }
    pub(super) fn set_textarea_default_value(
        &mut self,
        key: Key,
        raw: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.textarea_state(key)?;
        let (raw, payload) = source::formation::shared_string(self, raw, cancel)?;
        self.string_replace_all(key, raw, payload, limits, cancel)
    }
    pub(super) fn reset_textarea(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        self.textarea_state(key)?;
        self.with_textarea_state(key, cancel, |state| state.flags.reset())?;
        let (raw, payload) = self.textarea_child_text(key, limits, cancel)?;
        self.textarea_put_raw(key, raw, payload, limits, cancel)
    }
    pub(super) fn clone_textarea_overlay(
        &mut self,
        key: Key,
        original: &crate::DomTextareaState,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let units = (self.units - self.textarea_state(key)?.raw.len())
            .checked_add(original.raw.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let (raw, payload) = source::formation::shared_string(self, &original.raw, cancel)?;
        self.with_textarea_state_charged(key, cancel, payload, |state| {
            state.flags.clone_dirty_value_from(&original.flags);
            state.raw = raw;
        })?;
        self.units = units;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html::{
        HtmlInputEmptySelectionSupport as Empty, HtmlInputNoneDirectionSupport as NoneDirection,
        HtmlInputSelection, HtmlInputSelectionDirection as Direction, HtmlTextareaFlags,
    };

    #[test]
    fn normalized_text_reports_completed_units_and_the_prefix_before_refusal() {
        use super::super::scale_metrics::{Hooks, Measurement};
        if !super::super::scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        let raw = [u16::from(b'a'), 13, 10, u16::from(b'b')];
        let observation = Measurement::begin().unwrap();
        let normalized =
            crate::textarea::normalized_value(&raw, 16, &mut || Ok(()), None, &cancel).unwrap();
        let counters = observation.finish(Hooks::default()).unwrap();
        assert_eq!(
            normalized.code_units(),
            &[u16::from(b'a'), 10, u16::from(b'b')]
        );
        assert_eq!(counters.sequence_materialized_bytes, 3 * size_of::<u16>());

        let observation = Measurement::begin().unwrap();
        let mut steps = 0;
        let refused = crate::textarea::normalized_value(
            &raw,
            16,
            &mut || {
                steps += 1;
                // Four length reads, one allocation boundary, then the first
                // unit's read/write. Refuse before the second unit is copied.
                if steps == 8 {
                    Err(DomError::Cancelled)
                } else {
                    Ok(())
                }
            },
            None,
            &cancel,
        );
        assert_eq!(refused, Err(DomError::Cancelled));
        let counters = observation.finish(Hooks::default()).unwrap();
        assert_eq!(counters.sequence_materialized_bytes, size_of::<u16>());
    }

    #[test]
    fn normalized_setter_finish_replaces_a_surviving_range_and_child_clamp_preserves_direction() {
        for empty in [Empty::Supported, Empty::Unsupported] {
            for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
                let mut dom = Dom::with_native_input_platform(
                    DomLimits::new(
                        16,
                        2048,
                        32,
                        0,
                        0,
                        8,
                        65536,
                        crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)
                            .unwrap(),
                    )
                    .unwrap(),
                    crate::DomNativeInputPlatform::new(empty, none),
                )
                .unwrap();
                let cancel = AtomicBool::new(false);
                let mut b = DomBatch::new();
                let d = b.create_document();
                let textarea = b.create_element_ns(&d, crate::Namespace::html(), "textarea");
                b.set_textarea_default_value(&textarea, "A\r\nBC");
                b.set_custom_validity(&textarea, "error");
                let c = dom.apply(b, &cancel).unwrap();
                let textarea = c.created(&textarea).unwrap().unwrap();
                let source = dom.snapshot();
                let text = source.node(textarea).unwrap().children()[0];
                let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
                let key = draft.resolve(textarea.into()).unwrap();
                // A source-valid established range spans API offsets, not raw CRLF.
                draft
                    .with_textarea_state(key, &cancel, |state| {
                        state.selection =
                            HtmlInputSelection::selection(1, 3, Direction::Backward).unwrap()
                    })
                    .unwrap();
                draft
                    .set_textarea_value(key, &"A\nBC".into(), dom.limits, &cancel)
                    .unwrap();
                assert_eq!(
                    draft
                        .textarea_state(key)
                        .unwrap()
                        .selection
                        .selection_range(),
                    Some((1, 3, Direction::Backward))
                );
                draft
                    .set_textarea_value(key, &"WXYZ".into(), dom.limits, &cancel)
                    .unwrap();
                let selection = draft.textarea_state(key).unwrap().selection;
                assert_eq!(selection.cursor_position(), Some(4));
                assert_eq!(
                    selection.direction(),
                    if none == NoneDirection::Supported {
                        Direction::None
                    } else {
                        Direction::Forward
                    }
                );
                draft
                    .with_textarea_state(key, &cancel, |state| {
                        state.flags = HtmlTextareaFlags::new(true, true)
                    })
                    .unwrap();
                draft.reset_textarea(key, dom.limits, &cancel).unwrap();
                assert_eq!(
                    draft.textarea_state(key).unwrap().flags,
                    HtmlTextareaFlags::default()
                );
                assert_eq!(
                    draft.textarea_state(key).unwrap().control.message,
                    DocumentString::from("error")
                );
                draft
                    .with_textarea_state(key, &cancel, |state| {
                        state.selection =
                            HtmlInputSelection::selection(2, 4, Direction::Backward).unwrap()
                    })
                    .unwrap();
                draft
                    .apply(
                        &Command::ReplaceData {
                            node: text.into(),
                            offset: 0,
                            count: usize::MAX,
                            data: "A".into(),
                        },
                        dom.limits,
                        &cancel,
                    )
                    .unwrap();
                let state = draft.textarea_state(key).unwrap();
                assert_eq!(state.raw, DocumentString::from("A"));
                if empty == Empty::Supported {
                    assert_eq!(
                        state.selection.selection_range(),
                        Some((1, 1, Direction::Backward))
                    );
                } else {
                    assert_eq!(state.selection.cursor_position(), Some(1));
                }
                assert_eq!(state.selection.direction(), Direction::Backward);
            }
        }
    }

    #[test]
    fn prepared_real_raw_child_and_owner_prefix_never_publishes_after_cancel_or_exhaustion() {
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
            let textarea = b.create_element_ns(&d, crate::Namespace::html(), "textarea");
            b.insert_before(&form, &d, None);
            b.insert_before(&textarea, &form, None);
            b.set_textarea_default_value(&textarea, "initial");
            b.set_custom_validity(&textarea, "error");
            let c = dom.apply(b, &cancel).unwrap();
            let textarea = c.created(&textarea).unwrap().unwrap();
            let form = c.created(&form).unwrap().unwrap();
            assert_eq!(
                dom.snapshot()
                    .node(textarea)
                    .unwrap()
                    .as_textarea()
                    .unwrap()
                    .form_owner()
                    .unwrap(),
                Some(form)
            );
            if exhausted {
                dom.revision = crate::DomRevision(u64::MAX);
            }
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            let newdoc = b.create_document();
            b.set_textarea_value(textarea, "changed");
            b.set_textarea_default_value(textarea, "other\r\n");
            b.reset_textarea(textarea);
            b.adopt(textarea, &newdoc);
            let prepared = dom.prepare(b, &cancel).unwrap();
            let actual = prepared.state.node(dom.owner, textarea).unwrap();
            let DomNodeKind::Element(element) = &actual.kind else {
                unreachable!()
            };
            let state = element.textarea_state().unwrap();
            assert_eq!(state.raw_value(), &DocumentString::from("other\r\n"));
            assert!(!state.flags().dirty_value());
            assert_eq!(
                state.custom_validity_message(),
                &DocumentString::from("error")
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
