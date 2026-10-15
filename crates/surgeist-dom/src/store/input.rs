//! Source-ordered native Input transitions in the one canonical Draft.
use super::*;
use surgeist_html::{
    HtmlInputSelection, HtmlInputType, HtmlInputValueSanitization, HtmlRadioTrigger,
    KnownHtmlAttribute as A,
};

fn attribute_is(name: &crate::QualifiedName, expected: A) -> bool {
    name.namespace().uri().is_none()
        && name
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq(expected.local_name().encode_utf16())
}

fn equal(a: &DocumentString, b: &DocumentString, cancel: &AtomicBool) -> Result<bool, DomError> {
    let mut cursor =
        surgeist_html::HtmlInputValueComparisonCursor::new(a.code_units(), b.code_units());
    while cursor.required_work().is_some() {
        check_cancel(cancel)?;
        cursor.advance();
    }
    Ok(cursor.result().expect("completed comparison").is_equal())
}
impl Draft {
    pub(super) fn initialize_input(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.input_sanitize(key, limits, cancel)
    }
    pub(super) fn input_state(&self, key: Key) -> Result<&crate::DomInputState, DomError> {
        let DomNodeKind::Element(element) = &self.node(key)?.kind else {
            return Err(DomError::InputInapplicable);
        };
        element.input_state().ok_or(DomError::InputInapplicable)
    }
    fn input_state_mut(&mut self, key: Key) -> &mut crate::DomInputState {
        let DomNodeKind::Element(element) = &mut self.node_mut(key).kind else {
            unreachable!("checked Input")
        };
        element.input_state_mut().expect("checked Input")
    }
    fn input_install(
        &mut self,
        key: Key,
        state: crate::DomInputState,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        let old = self
            .input_state(key)?
            .units()
            .ok_or(DomError::RepresentationExhausted)?;
        let units = (self.units - old)
            .checked_add(state.units().ok_or(DomError::RepresentationExhausted)?)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        *self.input_state_mut(key) = state;
        self.units = units;
        Ok(())
    }
    fn input_raw(
        &self,
        key: Key,
        attribute: A,
        cancel: &AtomicBool,
    ) -> Result<Option<DocumentString>, DomError> {
        Ok(self.control_attribute(key, attribute, cancel)?.cloned())
    }
    fn input_put_value(
        &mut self,
        key: Key,
        value: DocumentString,
        email_count: Option<usize>,
        limits: DomLimits,
    ) -> Result<(), DomError> {
        let platform = self
            .native_environment(self.node(key)?.document)?
            .input_platform();
        let mut state = self.input_state(key)?.clone();
        if state.input_type().is_selectable() {
            state.selection = state
                .selection
                .clamp(value.len(), platform.empty_selection());
        }
        state.current = value;
        state.email_count = email_count;
        self.input_install(key, state, limits)
    }
    fn input_string_sanitize(
        &mut self,
        key: Key,
        raw: &DocumentString,
        policy: surgeist_html::HtmlInputSanitization,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::{HtmlInputEmailMode, HtmlInputSanitizationCursor as Cursor};
        let mode = if self.control_attribute(key, A::Multiple, cancel)?.is_some() {
            HtmlInputEmailMode::Multiple
        } else {
            HtmlInputEmailMode::Single
        };
        // Count exact selected output first. Raw input is borrowed caller or
        // already admitted immutable storage; no numeric/CSS allocation occurs.
        let mut cursor = Cursor::new(policy, raw.code_units(), mode);
        let mut count = 0usize;
        while cursor.required_work().is_some() {
            check_cancel(cancel)?;
            if let Some(range) = cursor.advance() {
                for &unit in &raw.code_units()[range.start()..range.end()] {
                    check_cancel(cancel)?;
                    if cursor.retain_unit(unit) {
                        count = count
                            .checked_add(1)
                            .ok_or(DomError::RepresentationExhausted)?;
                    }
                }
            }
        }
        let email_count = cursor.email_value_count();
        let total = (self.units - self.input_state(key)?.current.len())
            .checked_add(count)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(total, limits.units, DomResource::StringCodeUnits)?;
        let mut output = Vec::new();
        reserve(&mut output, count)?;
        let mut cursor = Cursor::new(policy, raw.code_units(), mode);
        while cursor.required_work().is_some() {
            check_cancel(cancel)?;
            if let Some(range) = cursor.advance() {
                for position in range.start()..range.end() {
                    check_cancel(cancel)?;
                    let unit = raw.code_units()[position];
                    if cursor.retain_unit(unit) {
                        output.push(cursor.replacement_unit(position).unwrap_or(unit));
                    }
                }
            }
        }
        self.input_put_value(key, DocumentString::from_utf16(output), email_count, limits)
    }
    fn input_sanitize(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let state = self.input_state(key)?;
        let raw = state.current.clone();
        match state.input_type().sanitization() {
            HtmlInputValueSanitization::String(policy) => {
                self.input_string_sanitize(key, &raw, policy, limits, cancel)
            }
            HtmlInputValueSanitization::Range => {
                let min = self.input_raw(key, A::Min, cancel)?;
                let max = self.input_raw(key, A::Max, cancel)?;
                let step = self.input_raw(key, A::Step, cancel)?;
                let default = self.input_raw(key, A::Value, cancel)?;
                let output = self.input_processing.range(
                    input_processing::RangeInput {
                        current: &raw,
                        min: min.as_ref(),
                        max: max.as_ref(),
                        step: step.as_ref(),
                        default_value: default.as_ref(),
                    },
                    cancel,
                )?;
                self.input_put_value(key, output, None, limits)
            }
            HtmlInputValueSanitization::Color => {
                use surgeist_html::{HtmlInputColorSpaceCursor, HtmlInputColorValueSource};
                let raw = match surgeist_html::input_color_value_source(
                    self.input_state(key)?.flags.dirty_value(),
                ) {
                    HtmlInputColorValueSource::CurrentValue => raw,
                    HtmlInputColorValueSource::ValueAttribute => {
                        self.input_raw(key, A::Value, cancel)?.unwrap_or_default()
                    }
                };
                let alpha = self.control_attribute(key, A::Alpha, cancel)?.is_some();
                let space = self.input_raw(key, A::Colorspace, cancel)?;
                let mut cursor = HtmlInputColorSpaceCursor::new(space.as_ref());
                while cursor.required_work().is_some() {
                    check_cancel(cancel)?;
                    cursor.advance();
                }
                let output = self.input_processing.color(
                    &raw,
                    alpha,
                    cursor.result().expect("completed colorspace"),
                    cancel,
                )?;
                self.input_put_value(key, output, None, limits)
            }
        }
    }
    pub(super) fn set_input_value(
        &mut self,
        key: Key,
        value: DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::{HtmlInputValueSetterStart, HtmlTextControlValueSetterFinish};
        check_cancel(cancel)?;
        match self.input_state(key)?.flags.value_setter_begin() {
            HtmlInputValueSetterStart::SetContentValue => self.set_attribute(
                key,
                Some(crate::Namespace::none()),
                "value".into(),
                value,
                limits,
                cancel,
            ),
            HtmlInputValueSetterStart::SetFilename => {
                let files = surgeist_html::input_filename_setter(value.code_units())
                    .map_err(|_| DomError::InputFilenameInvalidState)?;
                self.input_state_mut(key).files = files;
                Ok(())
            }
            HtmlInputValueSetterStart::SetCurrentValue(flags) => {
                let old = self.input_state(key)?.current.clone();
                self.input_state_mut(key).flags = flags;
                self.input_put_value(key, value, None, limits)?;
                self.input_sanitize(key, limits, cancel)?;
                let state = self.input_state(key)?;
                let mut comparison = surgeist_html::HtmlInputValueComparisonCursor::new(
                    old.code_units(),
                    state.current.code_units(),
                );
                while comparison.required_work().is_some() {
                    check_cancel(cancel)?;
                    comparison.advance();
                }
                if surgeist_html::input_value_setter_finish(
                    state.input_type(),
                    comparison.result().expect("completed comparison"),
                    state.selection,
                ) == HtmlTextControlValueSetterFinish::SetEndCursorUnselectAndNoneDirection
                {
                    let platform = self
                        .native_environment(self.node(key)?.document)?
                        .input_platform();
                    let len = state.current.len();
                    self.input_state_mut(key).selection = HtmlInputSelection::cursor(
                        len,
                        surgeist_html::HtmlInputSelectionDirection::None
                            .resolve_none(platform.none_direction()),
                    );
                }
                Ok(())
            }
        }
    }
    pub(super) fn set_input_checked(
        &mut self,
        key: Key,
        checked: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let flags = self.input_state(key)?.flags.checked_setter(checked);
        self.input_state_mut(key).flags = flags;
        self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)
    }
    pub(super) fn set_input_indeterminate(
        &mut self,
        key: Key,
        value: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let flags = self.input_state(key)?.flags.indeterminate_setter(value);
        self.input_state_mut(key).flags = flags;
        Ok(())
    }
    pub(super) fn reset_input(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::HtmlInputResetEffect as Effect;
        let mut cursor = surgeist_html::HtmlInputReset::new(self.input_state(key)?.flags);
        while let Some(effect) = cursor.next_effect() {
            check_cancel(cancel)?;
            match effect {
                Effect::ClearDirtyAndUserValidity(flags) => self.input_state_mut(key).flags = flags,
                Effect::ReplaceCurrentFromDefault => {
                    let raw = self.input_raw(key, A::Value, cancel)?.unwrap_or_default();
                    self.input_put_value(key, raw, None, limits)?;
                }
                Effect::SetCheckedFromPresence => {
                    let present = self.control_attribute(key, A::Checked, cancel)?.is_some();
                    let flags = self.input_state(key)?.flags.reset_checkedness(present);
                    self.input_state_mut(key).flags = flags;
                    self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)?;
                }
                Effect::EmptySelectedFiles => {
                    self.input_state_mut(key).files = surgeist_html::HtmlInputFileSelection::Empty
                }
                Effect::Sanitize(_) => self.input_sanitize(key, limits, cancel)?,
            }
        }
        Ok(())
    }
    fn input_type_changed(
        &mut self,
        key: Key,
        next: HtmlInputType,
        cause: attributes::AttributeCause,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::HtmlInputTypeEffect as Effect;
        let state = self.input_state(key)?;
        let previous = state.input_type();
        if previous == next {
            return Ok(());
        }
        let mut cursor =
            surgeist_html::HtmlInputTypeChange::new(previous, next, state.current.len());
        self.input_state_mut(key).flags = state.flags.type_changed(next);
        if next != HtmlInputType::Email {
            self.input_state_mut(key).email_count = None;
        }
        while let Some(effect) = cursor.next_effect() {
            check_cancel(cancel)?;
            match effect {
                Effect::SetContentValueFromCurrent => {
                    let raw = self.input_state(key)?.current.clone();
                    self.set_attribute(
                        key,
                        Some(crate::Namespace::none()),
                        "value".into(),
                        raw,
                        limits,
                        cancel,
                    )?;
                }
                Effect::SetCurrentFromDefaultAndClearDirty => {
                    let value = self.input_raw(key, A::Value, cancel)?.unwrap_or_default();
                    let flags = self.input_state(key)?.flags.type_default_installed();
                    self.input_state_mut(key).flags = flags;
                    self.input_put_value(key, value, None, limits)?;
                }
                Effect::SetCurrentEmptyForFilename => {
                    self.input_put_value(key, "".into(), None, limits)?
                }
                Effect::SetRendering => {} // Native closed profile has no rendering object.
                Effect::SignalTypeChange => {
                    self.input_radio_normalize(key, HtmlRadioTrigger::Type, cancel)?
                }
                Effect::Sanitize(_)
                    if next == HtmlInputType::Color
                        && matches!(cause, attributes::AttributeCause::CloneInputColor) => {}
                Effect::Sanitize(_) => self.input_sanitize(key, limits, cancel)?,
                Effect::SetInitialCursorAndNoneDirection => {
                    let platform = self
                        .native_environment(self.node(key)?.document)?
                        .input_platform();
                    self.input_state_mut(key).selection =
                        HtmlInputSelection::initial(platform.none_direction());
                }
            }
        }
        if next == HtmlInputType::Image {
            self.input_image_observe(key, true, limits, cancel)?;
        }
        Ok(())
    }
    fn input_image_observe(
        &mut self,
        key: Key,
        entering: bool,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let raw = self.input_raw(key, A::Src, cancel)?;
        let state = self.input_state(key)?;
        if entering
            && !state
                .image
                .entering_requires_observation(raw.as_ref(), |a, b| equal(a, b, cancel))?
        {
            return Ok(());
        }
        check_cancel(cancel)?;
        let guard = surgeist_html::HtmlInputImageUpdateGuard::for_source(raw.as_ref());
        let guard = if guard == surgeist_html::HtmlInputImageUpdateGuard::NeedsImageEnvironment {
            guard.with_disabled_images(
                self.native_environment(self.node(key)?.document)?
                    .input_images(),
            )
        } else {
            guard
        };
        debug_assert_ne!(
            guard,
            surgeist_html::HtmlInputImageUpdateGuard::NeedsImageEnvironment
        );
        let mut state = self.input_state(key)?.clone();
        state.image = state.image.with_observed_source(raw);
        self.input_install(key, state, limits)
    }
    pub(super) fn input_attribute_change(
        &mut self,
        key: Key,
        name: &crate::QualifiedName,
        change: attributes::AttributeChange,
        cause: attributes::AttributeCause,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Ok(state) = self.input_state(key) else {
            return Ok(());
        };
        let flags = state.flags;
        let cloned_color = flags.input_type() == HtmlInputType::Color
            && matches!(cause, attributes::AttributeCause::CloneInputColor);
        if !cloned_color
            && attribute_is(name, A::Value)
            && matches!(
                flags.value_attribute_changed(),
                surgeist_html::HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(_)
            )
        {
            let raw = self.input_raw(key, A::Value, cancel)?.unwrap_or_default();
            self.input_put_value(key, raw, None, limits)?;
            self.input_sanitize(key, limits, cancel)?;
        }
        if attribute_is(name, A::Checked) && !matches!(change, attributes::AttributeChange::Set) {
            let change = if matches!(change, attributes::AttributeChange::Removed) {
                surgeist_html::HtmlInputCheckedAttributeChange::Removed
            } else {
                surgeist_html::HtmlInputCheckedAttributeChange::Added
            };
            if let Some(next) = self
                .input_state(key)?
                .flags
                .checked_attribute_changed(change)
            {
                self.input_state_mut(key).flags = next;
                self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)?;
            }
        }
        if attribute_is(name, A::Type) {
            let raw = self.input_raw(key, A::Type, cancel)?;
            let mut cursor = surgeist_html::HtmlInputTypeCursor::new(raw.as_ref());
            while cursor.required_work().is_some() {
                check_cancel(cancel)?;
                cursor.advance();
            }
            self.input_type_changed(
                key,
                cursor.result().expect("completed actual type"),
                cause,
                limits,
                cancel,
            )?;
        }
        if attribute_is(name, A::Name) {
            self.input_radio_normalize(key, HtmlRadioTrigger::Name, cancel)?;
        }
        let kind = self.input_state(key)?.input_type();
        if kind == HtmlInputType::Email && attribute_is(name, A::Multiple) {
            self.input_sanitize(key, limits, cancel)?;
        }
        if kind == HtmlInputType::Range
            && [A::Min, A::Max, A::Step]
                .into_iter()
                .any(|a| attribute_is(name, a))
        {
            self.input_sanitize(key, limits, cancel)?;
        }
        if !cloned_color
            && kind == HtmlInputType::Color
            && [A::Alpha, A::Colorspace]
                .into_iter()
                .any(|a| attribute_is(name, a))
        {
            self.input_sanitize(key, limits, cancel)?;
        }
        if kind == HtmlInputType::Image && attribute_is(name, A::Src) {
            self.input_image_observe(key, false, limits, cancel)?;
        }
        Ok(())
    }
    pub(super) fn input_radio_normalize(
        &mut self,
        key: Key,
        trigger: HtmlRadioTrigger,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Ok(state) = self.input_state(key) else {
            return Ok(());
        };
        if !state.flags.radio_normalization_required(trigger) {
            return Ok(());
        }
        let Some(name) = self
            .input_raw(key, A::Name, cancel)?
            .filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        let root = crate::slot::root(self, key, cancel)?;
        let owner = self.node(key)?.control_owner;
        for index in 0..self.existing.len() + self.created.len() {
            check_cancel(cancel)?;
            let peer = if index < self.existing.len() {
                Key::Existing(index)
            } else {
                Key::Created(index - self.existing.len())
            };
            if peer == key {
                continue;
            }
            let Ok(node) = self.node(peer) else {
                continue;
            };
            let DomNodeKind::Element(element) = &node.kind else {
                continue;
            };
            let Some(state) = element.input_state() else {
                continue;
            };
            if state.input_type() != HtmlInputType::Radio
                || node.control_owner != owner
                || crate::slot::root(self, peer, cancel)? != root
            {
                continue;
            }
            let Some(other) = self.control_attribute(peer, A::Name, cancel)? else {
                continue;
            };
            if equal(&name, other, cancel)? {
                let flags = self.input_state(peer)?.flags.radio_peer_unchecked();
                self.input_state_mut(peer).flags = flags;
            }
        }
        Ok(())
    }
    pub(super) fn clone_input_overlay(
        &mut self,
        key: Key,
        original: &crate::DomInputState,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut state = self.input_state(key)?.clone();
        state.flags = state.flags.clone_flags_from(original.flags);
        state.current = original.current.clone();
        state.email_count = original.email_count;
        // Source copies the current value without generic sanitation/clamping;
        // selection/files/message/history/weak target retain fresh clone state.
        self.input_install(key, state, limits)?;
        self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reached_radio_peer_and_current_value_prefix_is_discarded_on_cancel_or_exhaustion() {
        for exhausted in [false, true] {
            let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 65536).unwrap()).unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let document = b.create_document();
            let root = b.create_element_ns(&document, crate::Namespace::html(), "div");
            let first = b.create_element_ns(&document, crate::Namespace::html(), "input");
            let second = b.create_element_ns(&document, crate::Namespace::html(), "input");
            b.insert_before(&root, &document, None);
            for node in [&first, &second] {
                b.set_attribute(node, "type", "radio");
                b.set_attribute(node, "name", "x");
                b.insert_before(node, &root, None);
            }
            b.set_input_checked(&first, true);
            let c = dom.apply(b, &cancel).unwrap();
            let first = c.created(&first).unwrap().unwrap();
            let second = c.created(&second).unwrap().unwrap();
            if exhausted {
                dom.revision = DomRevision(u64::MAX);
            }
            let before = dom.snapshot();
            let usage = dom.usage();
            let mut b = DomBatch::new();
            b.set_input_checked(second, true);
            b.set_input_value(second, "prefix");
            let prepared = dom.prepare(b, &cancel).unwrap();
            let first_state = prepared.state.node(dom.owner, first).unwrap();
            let DomNodeKind::Element(first_state) = &first_state.kind else {
                unreachable!()
            };
            assert!(!first_state.input_state().unwrap().flags().checkedness());
            let second_state = prepared.state.node(dom.owner, second).unwrap();
            let DomNodeKind::Element(second_state) = &second_state.kind else {
                unreachable!()
            };
            assert!(second_state.input_state().unwrap().flags().checkedness());
            assert_eq!(
                second_state.attribute(&"value".into()),
                Some(&"prefix".into())
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
    #[test]
    fn relevant_value_clamp_and_direction_use_the_independent_established_facts() {
        use surgeist_html::{
            HtmlInputEmptySelectionSupport as Empty,
            HtmlInputNoneDirectionSupport as NoneDirection,
            HtmlInputSelectionDirection as Direction,
        };
        for empty in [Empty::Supported, Empty::Unsupported] {
            for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
                let mut dom = Dom::with_native_input_platform(
                    DomLimits::new(16, 1024, 32, 0, 0, 4, 65536).unwrap(),
                    crate::DomNativeInputPlatform::new(empty, none),
                )
                .unwrap();
                let cancel = AtomicBool::new(false);
                let mut b = DomBatch::new();
                let d = b.create_document();
                let input = b.create_element_ns(&d, crate::Namespace::html(), "input");
                b.set_input_value(&input, "abcd");
                let c = dom.apply(b, &cancel).unwrap();
                let input = c.created(&input).unwrap().unwrap();
                let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
                let key = draft.resolve(input.into()).unwrap();
                // A valid established range fixture; public range/event operations
                // remain separate. Exercise the real native current-value path.
                draft.input_state_mut(key).selection =
                    HtmlInputSelection::selection(2, 4, Direction::Backward).unwrap();
                draft
                    .set_input_value(key, "ab".into(), dom.limits, &cancel)
                    .unwrap();
                let selection = draft.input_state(key).unwrap().selection;
                if empty == Empty::Supported {
                    assert_eq!(
                        selection.selection_range(),
                        Some((2, 2, Direction::Backward))
                    );
                    assert_eq!(selection.direction(), Direction::Backward);
                } else {
                    assert_eq!(selection.cursor_position(), Some(2));
                    assert_eq!(
                        selection.direction(),
                        if none == NoneDirection::Supported {
                            Direction::None
                        } else {
                            Direction::Forward
                        }
                    );
                }
            }
        }
    }
}
