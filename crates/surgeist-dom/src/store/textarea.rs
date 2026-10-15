//! Textarea's source-ordered raw, child, value, reset and clone transitions.
use super::*;

impl Draft {
    pub(super) fn textarea_state(&self, key: Key) -> Result<&crate::DomTextareaState, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Err(DomError::TextareaInapplicable);
        };
        element
            .textarea_state()
            .ok_or(DomError::TextareaInapplicable)
    }
    fn textarea_state_mut(&mut self, key: Key) -> &mut crate::DomTextareaState {
        let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
            unreachable!("checked Textarea")
        };
        element.textarea_state_mut().expect("checked Textarea")
    }
    fn textarea_install(
        &mut self,
        key: Key,
        state: crate::DomTextareaState,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        let old = self
            .textarea_state(key)?
            .units()
            .ok_or(DomError::RepresentationExhausted)?;
        let units = (self.units - old)
            .checked_add(state.units().ok_or(DomError::RepresentationExhausted)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        *self.textarea_state_mut(key) = state;
        self.units = units;
        Ok(())
    }
    fn textarea_put_raw(
        &mut self,
        key: Key,
        raw: DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let units = (self.units - self.textarea_state(key)?.raw.len())
            .checked_add(raw.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let length =
            crate::textarea::normalized_length(raw.code_units(), &mut || check_cancel(cancel))?;
        let platform = self
            .native_environment(self.node(key)?.document)?
            .input_platform();
        let mut state = self.textarea_state(key)?.clone();
        state.selection = state.selection.clamp(length, platform.empty_selection());
        state.raw = raw;
        self.textarea_install(key, state, limits)
    }
    fn textarea_child_text(
        &self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<DocumentString, DomError> {
        let mut length = 0usize;
        for &child in &self.node(key)?.children {
            check_cancel(cancel)?;
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
        length
            .checked_mul(std::mem::size_of::<u16>())
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(DomError::RepresentationExhausted)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(length)
            .map_err(|_| DomError::AllocationFailed)?;
        admit(
            output.capacity(),
            limits.units,
            DomResource::StringCodeUnits,
        )?;
        for &child in &self.node(key)?.children {
            check_cancel(cancel)?;
            if let DomNodeKind::Text(text) = &self.node(child)?.kind {
                for &unit in text.code_units() {
                    check_cancel(cancel)?;
                    output.push(unit);
                }
            }
        }
        Ok(DocumentString::from_utf16(output))
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
            let raw = self.textarea_child_text(key, limits, cancel)?;
            self.textarea_put_raw(key, raw, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn set_textarea_value(
        &mut self,
        key: Key,
        raw: DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let previous = self.textarea_state(key)?.raw.clone();
        let units = (self.units - previous.len())
            .checked_add(raw.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        // Both operands are admitted raw strings. Their normalized temporary
        // capacities are individually bounded by those actual raw lengths;
        // at most two coexist, <=2*the existing canonical unit ceiling.
        let old_api =
            crate::textarea::normalized_value(previous.code_units(), previous.len(), &mut || {
                check_cancel(cancel)
            })?;
        self.textarea_put_raw(key, raw, limits, cancel)?;
        self.textarea_state_mut(key).flags.mark_dirty_value();
        let current = &self.textarea_state(key)?.raw;
        let new_api =
            crate::textarea::normalized_value(current.code_units(), current.len(), &mut || {
                check_cancel(cancel)
            })?;
        let mut cursor = surgeist_html::HtmlInputValueComparisonCursor::new(
            old_api.code_units(),
            new_api.code_units(),
        );
        while cursor.required_work().is_some() {
            check_cancel(cancel)?;
            cursor.advance();
        }
        if surgeist_html::textarea_value_setter_finish(
            cursor.result().expect("completed actual API comparison"),
        )
            == surgeist_html::HtmlTextControlValueSetterFinish::SetEndCursorUnselectAndNoneDirection
        {
            let platform = self
                .native_environment(self.node(key)?.document)?
                .input_platform();
            self.textarea_state_mut(key).selection = surgeist_html::HtmlInputSelection::cursor(
                new_api.len(),
                surgeist_html::HtmlInputSelectionDirection::None
                    .resolve_none(platform.none_direction()),
            );
        }
        Ok(())
    }
    pub(super) fn set_textarea_default_value(
        &mut self,
        key: Key,
        raw: DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.textarea_state(key)?;
        self.string_replace_all(key, raw, limits, cancel)
    }
    pub(super) fn reset_textarea(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        self.textarea_state(key)?;
        self.textarea_state_mut(key).flags.reset();
        let raw = self.textarea_child_text(key, limits, cancel)?;
        self.textarea_put_raw(key, raw, limits, cancel)
    }
    pub(super) fn clone_textarea_overlay(
        &mut self,
        key: Key,
        original: &crate::DomTextareaState,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        let mut state = self.textarea_state(key)?.clone();
        state.flags.clone_dirty_value_from(&original.flags);
        state.raw = original.raw.clone();
        self.textarea_install(key, state, limits)
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
    fn normalized_setter_finish_replaces_a_surviving_range_and_child_clamp_preserves_direction() {
        for empty in [Empty::Supported, Empty::Unsupported] {
            for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
                let mut dom = Dom::with_native_input_platform(
                    DomLimits::new(16, 2048, 32, 0, 0, 8, 65536).unwrap(),
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
                draft.textarea_state_mut(key).selection =
                    HtmlInputSelection::selection(1, 3, Direction::Backward).unwrap();
                draft
                    .set_textarea_value(key, "A\nBC".into(), dom.limits, &cancel)
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
                    .set_textarea_value(key, "WXYZ".into(), dom.limits, &cancel)
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
                draft.textarea_state_mut(key).flags = HtmlTextareaFlags::new(true, true);
                draft.reset_textarea(key, dom.limits, &cancel).unwrap();
                assert_eq!(
                    draft.textarea_state(key).unwrap().flags,
                    HtmlTextareaFlags::default()
                );
                assert_eq!(
                    draft.textarea_state(key).unwrap().control.message,
                    DocumentString::from("error")
                );
                draft.textarea_state_mut(key).selection =
                    HtmlInputSelection::selection(2, 4, Direction::Backward).unwrap();
                draft
                    .apply(
                        Command::ReplaceData {
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
            let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 65536).unwrap()).unwrap();
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
                    .form_owner(),
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
