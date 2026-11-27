//! Source-ordered native Input transitions in the one canonical Draft.
use super::*;
use surgeist_html::{
    accounting::{HtmlChange, HtmlResource, HtmlWork},
    attributes::HtmlInputType,
    elements::input::{HtmlInputSelection, HtmlInputValueSanitization, HtmlRadioTrigger},
    vocabulary::KnownHtmlAttribute as A,
};
use surgeist_ledger::Charge;

use super::source::operation::element_queue::{DomFileSelectionTask, Kind, Pending};

/// The DOM's meter for the HTML input cursors: one source step for each cursor action, whatever
/// work the cursor describes. The cursors charge through it before each action they perform.
pub(super) struct StepMeter<'a>(pub(super) &'a mut dyn FnMut() -> Result<(), DomError>);

impl Charge<HtmlWork, HtmlResource> for StepMeter<'_> {
    type Stop = DomError;

    fn charge(&mut self, _: &[HtmlChange]) -> Result<(), DomError> {
        (self.0)()
    }

    fn record_largest(&mut self, _: HtmlResource, _: u64) -> Result<(), DomError> {
        Err(DomError::HtmlChargeProtocol)
    }
}

impl Draft {
    pub(super) fn eligible_user_file_selection(
        &self,
        key: Key,
        files: &files::DomFileList,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        match self.input_state(key) {
            Ok(state) if state.input_type() == HtmlInputType::File => {}
            Ok(_) | Err(DomError::InputInapplicable | DomError::StaleNode) => return Ok(false),
            Err(error) => return Err(error),
        }
        Ok(!self.control_is_disabled(key, cancel)?
            && (files.len() <= 1 || self.control_attribute(key, A::Multiple, cancel)?.is_some()))
    }

    pub(super) fn queue_user_file_selection(
        &mut self,
        key: Key,
        files: &files::DomFileList,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if !self.eligible_user_file_selection(key, files, cancel)? {
            return Err(DomError::FileSelectionInapplicable);
        }
        let association = self
            .element_task_association(key, cancel)?
            .ok_or(DomError::FileSelectionTaskSourceUnavailable)?;
        self.queue_final_element_task(
            Pending {
                target: key,
                association,
                kind: Kind::FileSelection,
            },
            cancel,
        )
    }

    pub(super) fn assign_input_files(
        &mut self,
        key: Key,
        files: Option<&files::DomFileList>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let state = self.input_state(key)?;
        let Some(files) = files.filter(|_| state.input_type() == HtmlInputType::File) else {
            return Ok(());
        };
        if state.selected_files() != Some(files) {
            self.with_input_state(key, cancel, |state| state.files = Some(files.clone()))?;
        }
        Ok(())
    }

    fn empty_input_files(
        &self,
        cancel: &AtomicBool,
    ) -> Result<(files::DomFileList, bank::ObjectCustody), DomError> {
        let budget = source::formation::Formation::new(self, cancel)?;
        files::DomFileList::controlled_empty(
            self.custody_registry(),
            budget.as_ref(),
            &mut |usage| self.admit_custody_usage(usage, cancel),
        )
    }

    fn clear_input_files(&mut self, key: Key, cancel: &AtomicBool) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self
            .input_state(key)?
            .selected_files()
            .is_some_and(|files| !files.is_empty())
        {
            let (empty, _control) = self.empty_input_files(cancel)?;
            self.with_input_state(key, cancel, |state| state.files = Some(empty))?;
        }
        Ok(())
    }

    pub(super) fn step_input(
        &mut self,
        key: Key,
        direction: surgeist_html::elements::input::HtmlInputStepDirection,
        count: i32,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use input_processing::{InputProcessing, StepInput, StepRead};

        self.source_step(cancel)?;
        let kind = self.input_state(key)?.input_type();
        if !kind.supports_value_as_number() {
            return Err(DomError::InputStepInvalidState);
        }
        let source = source::formation::Formation::new(self, cancel)?;
        // Each numeric stage requests the actual canonical operand only
        // when reached. Keep its original meter outside that local borrow.
        let mut processing =
            std::mem::replace(&mut self.input_processing, InputProcessing::new(0, 0));
        let result = processing.step(
            StepInput {
                kind,
                direction,
                count,
            },
            source.as_ref(),
            |read| match read {
                StepRead::Step => self.input_raw(key, A::Step, cancel),
                StepRead::Minimum => self.input_raw(key, A::Min, cancel),
                StepRead::Maximum => self.input_raw(key, A::Max, cancel),
                StepRead::Default => self.input_raw(key, A::Value, cancel),
                StepRead::Current => self
                    .input_shared(&self.input_state(key)?.current, cancel)
                    .map(Some),
            },
            cancel,
        );
        self.input_processing = processing;
        if let Some(value) = result? {
            self.input_put_value(key, value, None, limits, cancel)?;
        }
        Ok(())
    }

    fn input_attribute_is(
        &self,
        name: &crate::internal::name::DomQualifiedName,
        expected: A,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        Ok(name.namespace_uri().is_none()
            && self.source_ascii(name.local_name(), expected.local_name(), cancel)?)
    }
    fn input_equal(
        &self,
        a: &DocumentString,
        b: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let mut cursor = surgeist_html::elements::input::HtmlInputValueComparisonCursor::new(
            a.code_units(),
            b.code_units(),
        );
        let mut steps = StepMeter(&mut || self.source_step(cancel));
        while cursor.result().is_none() {
            cursor.advance(&mut steps)?;
        }
        self.source_step(cancel)?;
        Ok(cursor.result().expect("completed comparison").is_equal())
    }
    fn input_shared(
        &self,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<input_processing::InputValue, DomError> {
        self.source_step(cancel)?;
        let (value, payload) = source::formation::shared_string(self, value, cancel)?;
        Ok(input_processing::InputValue { value, payload })
    }
    fn input_empty(&self, cancel: &AtomicBool) -> Result<input_processing::InputValue, DomError> {
        let formation = source::formation::Formation::new(self, cancel)?;
        let value = source::formation::string("", formation.as_ref(), cancel)?;
        Ok(input_processing::InputValue {
            value,
            payload: formation.map(source::formation::Formation::into_charge),
        })
    }
    pub(super) fn initialize_input(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.input_sanitize(key, limits, cancel)
    }
    pub(super) fn input_state(
        &self,
        key: Key,
    ) -> Result<&crate::internal::input::DomInputState, DomError> {
        let transaction::KindRead::Element(element) = self.node(key)?.kind() else {
            return Err(DomError::InputInapplicable);
        };
        element
            .language()
            .input_state()
            .ok_or(DomError::InputInapplicable)
    }
    pub(super) fn with_input_state<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        edit: impl FnOnce(&mut crate::internal::input::DomInputState) -> R,
    ) -> Result<R, DomError> {
        let mut state = self.input_state_mut(key, cancel)?;
        let result = edit(&mut state);
        state.commit(cancel)?;
        Ok(result)
    }
    fn with_input_state_charged<R>(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
        payload: Option<source::checkpoint::Charge>,
        edit: impl FnOnce(&mut crate::internal::input::DomInputState) -> R,
    ) -> Result<R, DomError> {
        let mut state = self.input_state_mut(key, cancel)?;
        let result = edit(&mut state);
        state.commit_charged(payload, cancel)?;
        Ok(result)
    }
    fn input_raw(
        &self,
        key: Key,
        attribute: A,
        cancel: &AtomicBool,
    ) -> Result<Option<input_processing::InputValue>, DomError> {
        self.control_attribute(key, attribute, cancel)?
            .map(|value| self.input_attribute_value(value, cancel))
            .transpose()
    }
    /// The value of an attribute that the input retains, as its current value or a derived one.
    /// The input state prices its strings by the capacity of their blocks, whichever path set
    /// them, so a value that borrows a block is copied into an exact owner here instead of
    /// keeping that block alive at the price of its capacity.
    fn input_attribute_value(
        &self,
        value: &DocumentString,
        cancel: &AtomicBool,
    ) -> Result<input_processing::InputValue, DomError> {
        if value.borrowed_block().is_none() {
            return self.input_shared(value, cancel);
        }
        self.source_step(cancel)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let value =
            source::formation::units(value.code_units(), false, formation.as_ref(), cancel)?;
        Ok(input_processing::InputValue {
            value,
            payload: formation.map(source::formation::Formation::into_charge),
        })
    }
    fn input_default(
        &self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<input_processing::InputValue, DomError> {
        match self.input_raw(key, A::Value, cancel)? {
            Some(value) => Ok(value),
            None => self.input_empty(cancel),
        }
    }
    pub(super) fn input_put_value(
        &mut self,
        key: Key,
        value: input_processing::InputValue,
        email_count: Option<usize>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let platform = self
            .document_metadata(self.node(key)?.document())?
            .document_environment()
            .input_platform();
        let units = (self.units - self.input_state(key)?.current.len())
            .checked_add(value.value.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let previous = self.input_state(key)?;
        let selection = if previous.input_type().is_selectable() {
            previous
                .selection
                .clamp(value.value.len(), platform.empty_selection())
        } else {
            previous.selection
        };
        self.with_input_state_charged(key, cancel, value.payload, |state| {
            state.selection = selection;
            state.current = value.value;
            state.email_count = email_count;
        })?;
        self.units = units;
        Ok(())
    }
    fn input_string_sanitize(
        &mut self,
        key: Key,
        raw: &DocumentString,
        policy: surgeist_html::elements::input::HtmlInputSanitization,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::elements::input::{
            HtmlInputEmailMode, HtmlInputSanitizationCursor as Cursor,
        };
        let mode = if self.control_attribute(key, A::Multiple, cancel)?.is_some() {
            HtmlInputEmailMode::Multiple
        } else {
            HtmlInputEmailMode::Single
        };
        // Count exact selected output first. Raw input is borrowed caller or
        // already admitted immutable storage; no numeric/CSS allocation occurs.
        let mut cursor = Cursor::new(policy, raw.code_units(), mode);
        let mut count = 0usize;
        let mut steps = StepMeter(&mut || self.source_step(cancel));
        while !cursor.is_complete() {
            if let Some(range) = cursor.advance(&mut steps)? {
                for unit in &raw.code_units()[range.start()..range.end()] {
                    self.source_step(cancel)?;
                    if cursor.retain_unit(*unit) {
                        count = count
                            .checked_add(1)
                            .ok_or(DomError::RepresentationExhausted)?;
                    }
                }
            }
        }
        self.source_step(cancel)?;
        let email_count = cursor.email_value_count();
        let total = (self.units - self.input_state(key)?.current.len())
            .checked_add(count)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(total, limits.units, DomResource::StringCodeUnits)?;
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::InputString,
            cancel,
        )?;
        let formation = source::formation::Formation::new(self, cancel)?;
        #[cfg(test)]
        let mut output_writes = crate::internal::store::CopyObservation::default();
        let mut output = match &formation {
            Some(formation) => {
                source::formation::utf16_buffer(count, limits.units, Some(formation), cancel)?
            }
            None => {
                let mut output = Vec::new();
                reserve(&mut output, count)?;
                output
            }
        };
        let mut cursor = Cursor::new(policy, raw.code_units(), mode);
        let mut steps = StepMeter(&mut || self.source_step(cancel));
        while !cursor.is_complete() {
            if let Some(range) = cursor.advance(&mut steps)? {
                for position in range.start()..range.end() {
                    self.source_step(cancel)?;
                    let unit = raw.code_units()[position];
                    if cursor.retain_unit(unit) {
                        output.push(cursor.replacement_unit(position).unwrap_or(unit));
                        #[cfg(test)]
                        output_writes.pushed::<u16>();
                    }
                }
            }
        }
        self.source_step(cancel)?;
        let output = DocumentString::from_utf16(output);
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            size_of::<Vec<u16>>() + output.retained_units() * size_of::<u16>(),
        );
        self.input_put_value(
            key,
            input_processing::InputValue {
                value: output,
                payload: formation.map(source::formation::Formation::into_charge),
            },
            email_count,
            limits,
            cancel,
        )
    }
    fn input_sanitize(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let input_type = self.input_state(key)?.input_type();
        #[cfg(test)]
        let growth = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::InputAlias,
            cancel,
        )?;
        let raw = self.input_shared(&self.input_state(key)?.current, cancel)?;
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            growth,
            size_of::<Vec<u16>>() + raw.value.retained_units() * size_of::<u16>(),
        );
        match input_type.sanitization() {
            HtmlInputValueSanitization::String(policy) => {
                self.input_string_sanitize(key, &raw.value, policy, limits, cancel)
            }
            HtmlInputValueSanitization::Range => {
                let min = self.input_raw(key, A::Min, cancel)?;
                let max = self.input_raw(key, A::Max, cancel)?;
                let step = self.input_raw(key, A::Step, cancel)?;
                let default = self.input_raw(key, A::Value, cancel)?;
                #[cfg(test)]
                let _source_observation = input_processing::source_observation::bind(self);
                let source = source::formation::Formation::new(self, cancel)?;
                let output = self.input_processing.range(
                    input_processing::RangeInput {
                        current: &raw.value,
                        min: min.as_ref().map(|raw| &raw.value),
                        max: max.as_ref().map(|raw| &raw.value),
                        step: step.as_ref().map(|raw| &raw.value),
                        default_value: default.as_ref().map(|raw| &raw.value),
                    },
                    source.as_ref(),
                    cancel,
                )?;
                self.input_put_value(key, output, None, limits, cancel)
            }
            HtmlInputValueSanitization::Color => {
                use surgeist_html::elements::input::{
                    HtmlInputColorSpaceCursor, HtmlInputColorValueSource,
                };
                let raw = match surgeist_html::elements::input::input_color_value_source(
                    self.input_state(key)?.flags.dirty_value(),
                ) {
                    HtmlInputColorValueSource::CurrentValue => raw,
                    HtmlInputColorValueSource::ValueAttribute => {
                        drop(raw);
                        self.input_default(key, cancel)?
                    }
                };
                let alpha = self.control_attribute(key, A::Alpha, cancel)?.is_some();
                let space = self.input_raw(key, A::Colorspace, cancel)?;
                let mut cursor = HtmlInputColorSpaceCursor::new(
                    space.as_ref().map(|raw| raw.value.code_units()),
                );
                let mut steps = StepMeter(&mut || self.source_step(cancel));
                while cursor.result().is_none() {
                    cursor.advance(&mut steps)?;
                }
                self.source_step(cancel)?;
                #[cfg(test)]
                let _source_observation = input_processing::source_observation::bind(self);
                let source = source::formation::Formation::new(self, cancel)?;
                let output = self.input_processing.color(
                    &raw.value,
                    alpha,
                    cursor.result().expect("completed colorspace"),
                    source.as_ref(),
                    cancel,
                )?;
                self.input_put_value(key, output, None, limits, cancel)
            }
        }
    }
    pub(super) fn set_input_value(
        &mut self,
        key: Key,
        value: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::elements::{
            form::HtmlTextControlValueSetterFinish, input::HtmlInputValueSetterStart,
        };
        self.source_step(cancel)?;
        match self.input_state(key)?.flags.value_setter_begin() {
            HtmlInputValueSetterStart::SetContentValue => {
                let operand = source::formation::Formation::new(self, cancel)?;
                let name = source::formation::string("value", operand.as_ref(), cancel)?;
                self.set_attribute(
                    key,
                    Some(&surgeist_html::values::Namespace::none()),
                    &name,
                    value,
                    limits,
                    cancel,
                )
            }
            HtmlInputValueSetterStart::SetFilename => {
                self.source_step(cancel)?;
                surgeist_html::elements::input::input_filename_setter(value.code_units())
                    .map_err(|_| DomError::InputFilenameInvalidState)?;
                self.clear_input_files(key, cancel)
            }
            HtmlInputValueSetterStart::SetCurrentValue(flags) => {
                let old = self.input_shared(&self.input_state(key)?.current, cancel)?;
                self.with_input_state(key, cancel, |state| state.flags = flags)?;
                self.input_put_value(key, self.input_shared(value, cancel)?, None, limits, cancel)?;
                self.input_sanitize(key, limits, cancel)?;
                let state = self.input_state(key)?;
                let mut comparison =
                    surgeist_html::elements::input::HtmlInputValueComparisonCursor::new(
                        old.value.code_units(),
                        state.current.code_units(),
                    );
                let mut steps = StepMeter(&mut || self.source_step(cancel));
                while comparison.result().is_none() {
                    comparison.advance(&mut steps)?;
                }
                self.source_step(cancel)?;
                if surgeist_html::elements::input::input_value_setter_finish(
                    state.input_type(),
                    comparison.result().expect("completed comparison"),
                    state.selection,
                ) == HtmlTextControlValueSetterFinish::SetEndCursorUnselectAndNoneDirection
                {
                    let platform = self
                        .document_metadata(self.node(key)?.document())?
                        .document_environment()
                        .input_platform();
                    let len = state.current.len();
                    self.with_input_state(key, cancel, |state| {
                        state.selection = HtmlInputSelection::cursor(
                            len,
                            surgeist_html::elements::input::HtmlInputSelectionDirection::None
                                .resolve_none(platform.none_direction()),
                        )
                    })?;
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
        self.source_step(cancel)?;
        let flags = self.input_state(key)?.flags.checked_setter(checked);
        self.with_input_state(key, cancel, |state| state.flags = flags)?;
        self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)
    }
    pub(super) fn select_radio_from_node_list(
        &mut self,
        key: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let flags = self
            .input_state(key)?
            .flags
            .radio_node_list_selected()
            .ok_or(DomError::InputInapplicable)?;
        self.with_input_state(key, cancel, |state| state.flags = flags)?;
        self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)
    }
    pub(super) fn set_input_indeterminate(
        &mut self,
        key: Key,
        value: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let flags = self.input_state(key)?.flags.indeterminate_setter(value);
        self.with_input_state(key, cancel, |state| state.flags = flags)?;
        Ok(())
    }
    pub(super) fn reset_input(
        &mut self,
        key: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use surgeist_html::elements::input::HtmlInputResetEffect as Effect;
        let mut cursor =
            surgeist_html::elements::input::HtmlInputReset::new(self.input_state(key)?.flags);
        loop {
            self.source_step(cancel)?;
            let Some(effect) = cursor.next_effect() else {
                break;
            };
            match effect {
                Effect::ClearDirtyAndUserValidity(flags) => {
                    self.with_input_state(key, cancel, |state| state.flags = flags)?;
                }
                Effect::ReplaceCurrentFromDefault => {
                    let raw = self.input_default(key, cancel)?;
                    self.input_put_value(key, raw, None, limits, cancel)?;
                }
                Effect::SetCheckedFromPresence => {
                    let present = self.control_attribute(key, A::Checked, cancel)?.is_some();
                    let flags = self.input_state(key)?.flags.reset_checkedness(present);
                    self.with_input_state(key, cancel, |state| state.flags = flags)?;
                    self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)?;
                }
                Effect::EmptySelectedFiles => self.clear_input_files(key, cancel)?,
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
        use surgeist_html::elements::input::HtmlInputTypeEffect as Effect;
        let state = self.input_state(key)?;
        let previous = state.input_type();
        if previous == next {
            return Ok(());
        }
        let mut cursor = surgeist_html::elements::input::HtmlInputTypeChange::new(
            previous,
            next,
            state.current.len(),
        );
        let flags = state.flags.type_changed(next);
        let (files, _control) = if next == HtmlInputType::File {
            let (files, control) = self.empty_input_files(cancel)?;
            (Some(files), Some(control))
        } else {
            (None, None)
        };
        self.with_input_state(key, cancel, |state| {
            state.flags = flags;
            state.files = files;
        })?;
        if next != HtmlInputType::Email {
            self.with_input_state(key, cancel, |state| state.email_count = None)?;
        }
        loop {
            self.source_step(cancel)?;
            let Some(effect) = cursor.next_effect() else {
                break;
            };
            match effect {
                Effect::SetContentValueFromCurrent => {
                    let operand = source::formation::Formation::new(self, cancel)?;
                    let raw = source::formation::share(
                        &self.input_state(key)?.current,
                        operand.as_ref(),
                        cancel,
                    )?;
                    let name = source::formation::string("value", operand.as_ref(), cancel)?;
                    self.set_attribute(
                        key,
                        Some(&surgeist_html::values::Namespace::none()),
                        &name,
                        &raw,
                        limits,
                        cancel,
                    )?;
                }
                Effect::SetCurrentFromDefaultAndClearDirty => {
                    let value = self.input_default(key, cancel)?;
                    let flags = self.input_state(key)?.flags.type_default_installed();
                    self.with_input_state(key, cancel, |state| state.flags = flags)?;
                    self.input_put_value(key, value, None, limits, cancel)?;
                }
                Effect::SetCurrentEmptyForFilename => {
                    self.input_put_value(key, self.input_empty(cancel)?, None, limits, cancel)?
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
                        .document_metadata(self.node(key)?.document())?
                        .document_environment()
                        .input_platform();
                    self.with_input_state(key, cancel, |state| {
                        state.selection = HtmlInputSelection::initial(platform.none_direction())
                    })?;
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
                .entering_requires_observation(raw.as_ref().map(|raw| &raw.value), |a, b| {
                    self.input_equal(a, b, cancel)
                })?
        {
            return Ok(());
        }
        self.source_step(cancel)?;
        let guard = surgeist_html::elements::input::HtmlInputImageUpdateGuard::for_source(
            raw.as_ref().map(|raw| raw.value.code_units()),
        );
        let guard = if guard
            == surgeist_html::elements::input::HtmlInputImageUpdateGuard::NeedsImageEnvironment
        {
            guard.with_disabled_images(
                self.document_metadata(self.node(key)?.document())?
                    .document_environment()
                    .input_images(),
            )
        } else {
            guard
        };
        debug_assert_ne!(
            guard,
            surgeist_html::elements::input::HtmlInputImageUpdateGuard::NeedsImageEnvironment
        );
        let old = self
            .input_state(key)?
            .image
            .observed_source()
            .flatten()
            .map_or(0, DocumentString::len);
        let next = raw.as_ref().map_or(0, |raw| raw.value.len());
        let units = (self.units - old)
            .checked_add(next)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        let (raw, payload) = match raw {
            Some(input_processing::InputValue { value, payload }) => (Some(value), payload),
            None => (None, None),
        };
        self.with_input_state_charged(key, cancel, payload, |state| {
            state.image = state.image.with_observed_source(raw)
        })?;
        self.units = units;
        Ok(())
    }
    pub(super) fn input_attribute_change(
        &mut self,
        key: Key,
        name: &crate::internal::name::DomQualifiedName,
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
            && self.input_attribute_is(name, A::Value, cancel)?
            && matches!(
                flags.value_attribute_changed(),
                surgeist_html::elements::input::HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(_)
            )
        {
            let raw = self.input_default(key, cancel)?;
            self.input_put_value(key, raw, None, limits, cancel)?;
            self.input_sanitize(key, limits, cancel)?;
        }
        if self.input_attribute_is(name, A::Checked, cancel)?
            && !matches!(change, attributes::AttributeChange::Set)
        {
            let change = if matches!(change, attributes::AttributeChange::Removed) {
                surgeist_html::elements::input::HtmlInputCheckedAttributeChange::Removed
            } else {
                surgeist_html::elements::input::HtmlInputCheckedAttributeChange::Added
            };
            if let Some(next) = self
                .input_state(key)?
                .flags
                .checked_attribute_changed(change)
            {
                self.with_input_state(key, cancel, |state| state.flags = next)?;
                self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)?;
            }
        }
        if self.input_attribute_is(name, A::Type, cancel)? {
            let raw = self.input_raw(key, A::Type, cancel)?;
            let mut cursor = surgeist_html::elements::input::HtmlInputTypeCursor::new(
                raw.as_ref().map(|raw| raw.value.code_units()),
            );
            let mut steps = StepMeter(&mut || self.source_step(cancel));
            while cursor.result().is_none() {
                cursor.advance(&mut steps)?;
            }
            self.source_step(cancel)?;
            self.input_type_changed(
                key,
                cursor.result().expect("completed actual type"),
                cause,
                limits,
                cancel,
            )?;
        }
        if self.input_attribute_is(name, A::Name, cancel)? {
            self.input_radio_normalize(key, HtmlRadioTrigger::Name, cancel)?;
        }
        let kind = self.input_state(key)?.input_type();
        if kind == HtmlInputType::Email && self.input_attribute_is(name, A::Multiple, cancel)? {
            self.input_sanitize(key, limits, cancel)?;
        }
        if kind == HtmlInputType::Range
            && (self.input_attribute_is(name, A::Min, cancel)?
                || self.input_attribute_is(name, A::Max, cancel)?
                || self.input_attribute_is(name, A::Step, cancel)?)
        {
            self.input_sanitize(key, limits, cancel)?;
        }
        if !cloned_color
            && kind == HtmlInputType::Color
            && (self.input_attribute_is(name, A::Alpha, cancel)?
                || self.input_attribute_is(name, A::Colorspace, cancel)?)
        {
            self.input_sanitize(key, limits, cancel)?;
        }
        if kind == HtmlInputType::Image && self.input_attribute_is(name, A::Src, cancel)? {
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
        let key = self.live_key(key);
        let Ok(state) = self.input_state(key) else {
            return Ok(());
        };
        if !state.flags.radio_normalization_required(trigger) {
            return Ok(());
        }
        #[cfg(test)]
        let alias = source::formation::mutation_observation::before(
            self,
            source::formation::mutation_observation::Site::InputRadioName,
            cancel,
        )?;
        let Some(name) = self
            .input_raw(key, A::Name, cancel)?
            .filter(|value| !value.value.is_empty())
        else {
            return Ok(());
        };
        #[cfg(test)]
        source::formation::mutation_observation::grew(
            alias,
            size_of::<Vec<u16>>() + name.value.retained_units() * size_of::<u16>(),
        );
        #[cfg(test)]
        super::control::participant_tests::radio_discovery_started(self);
        let peers = self.control_radio_participants(key, &name.value, cancel)?;
        let mut selected = peers.cursor(|| self.source_step(cancel))?;
        while let Some(peer) = selected.next(|| self.source_step(cancel))? {
            self.source_step(cancel)?;
            let peer = self.live_key(peer);
            if peer == key {
                continue;
            }
            let flags = self.input_state(peer)?.flags.radio_peer_unchecked();
            self.with_input_state(peer, cancel, |state| state.flags = flags)?;
        }
        Ok(())
    }
    pub(super) fn clone_input_overlay(
        &mut self,
        key: Key,
        original: &crate::internal::input::DomInputState,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let current = self.input_shared(&original.current, cancel)?;
        let units = (self.units - self.input_state(key)?.current.len())
            .checked_add(current.value.len())
            .ok_or(DomError::RepresentationExhausted)?;
        admit(units, limits.units, DomResource::StringCodeUnits)?;
        self.with_input_state_charged(key, cancel, current.payload, |state| {
            state.flags = state.flags.clone_flags_from(original.flags);
            state.current = current.value;
            state.email_count = original.email_count;
        })?;
        // Only the original current/flags overlay is copied; all other fields
        // remain the freshly initialized target's actual values.
        self.units = units;
        self.input_radio_normalize(key, HtmlRadioTrigger::Checkedness, cancel)
    }
}

impl Dom {
    /// Apply the original user file selection once, under the task-bank allowance.
    /// A retired or ineligible target consumes the permit without installing
    /// anything. A refusal leaves the permit live and debits actual work.
    pub fn run_file_selection_task(
        &mut self,
        task: &mut Option<DomFileSelectionTask>,
        mut limits: crate::internal::source::DomSourceLimits,
        bank_work: &mut u64,
        cancel: &AtomicBool,
    ) -> Result<Option<DomCommit>, DomError> {
        limits.work = limits.work.min(*bank_work);
        let initial = source::initial::Initial::new(limits);
        let formed = (|| {
            initial.step(cancel)?;
            self.require_sync_source_idle()?;
            let permit = task.as_ref().ok_or(DomError::InvalidNativeTask)?;
            self.qualify_element_task(&permit.binding)?;
            initial.retain(
                source::storage::Payload {
                    bytes: size_of::<BatchIdentity>(),
                    units: 0,
                },
                cancel,
            )?;
            let draft = Draft::new_formed(
                self,
                source::formation::allocated_arc(BatchIdentity),
                Some(&initial),
                cancel,
            )?;
            Ok((draft, permit))
        })();
        let (mut draft, permit) = match formed {
            Ok(formed) => formed,
            Err(error) => {
                *bank_work -= initial.spent_work();
                return Err(error);
            }
        };
        let result = (|| {
            let info = permit.info();
            let key = Key::Existing(info.id().target());
            if !draft.eligible_user_file_selection(key, &permit.files, cancel)?
                || draft.element_task_association(key, cancel)? != Some(info.queue_association())
            {
                return Ok(None);
            }
            draft.assign_input_files(key, Some(&permit.files), cancel)?;
            let prepared = draft.finish(self, cancel)?;
            if limits.phases == 0 && prepared.changed_from(&self.state) {
                return Err(DomError::SourcePublicationLimit);
            }
            if limits.outputs == 0 {
                return Err(DomError::SourceOutputLimit);
            }
            self.publish(prepared, cancel).map(Some)
        })();
        *bank_work -= draft.source_work.load(Ordering::Relaxed);
        let commit = result?;
        task.take();
        Ok(commit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_radio_name(bytes: usize) -> Option<source::formation::mutation_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                16,
                1024,
                32,
                0,
                0,
                8,
                100_000,
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent =
            setup.create_element_ns(&document, surgeist_html::values::Namespace::html(), "div");
        let key =
            setup.create_element_ns(&document, surgeist_html::values::Namespace::html(), "input");
        let peer =
            setup.create_element_ns(&document, surgeist_html::values::Namespace::html(), "input");
        for input in [&key, &peer] {
            setup.set_attribute(input, "type", "radio");
            setup.insert_before(input, &parent, None);
        }
        let mut padded = Vec::with_capacity(65_536);
        padded.push(u16::from(b'g'));
        setup.set_attribute(&key, "name", DocumentString::from_utf16(padded));
        setup.set_attribute(&peer, "name", "g");
        setup.set_input_checked(&peer, true);
        setup.insert_before(&parent, &document, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let key = created.created(&key).unwrap().unwrap();
        let peer = created.created(&peer).unwrap().unwrap();
        let before = dom.snapshot();
        assert!(
            !before
                .node(key)
                .unwrap()
                .as_input()
                .unwrap()
                .state()
                .flags()
                .checkedness()
        );
        assert!(
            before
                .node(peer)
                .unwrap()
                .as_input()
                .unwrap()
                .state()
                .flags()
                .checkedness()
        );
        let mut batch = DomBatch::new();
        batch.set_input_checked(key, true);
        let mut lease = match dom.begin_source_commands(
            batch,
            crate::internal::source::DomSourceLimits::new(
                bytes,
                1_000_000,
                100_000_000,
                64,
                4096,
                4096,
                1,
            ),
            &cancel,
        ) {
            Ok(lease) => lease,
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            }) => return None,
            Err(error) => panic!("actual Radio Source admission: {error:?}"),
        };
        source::formation::mutation_observation::start(
            source::formation::mutation_observation::Site::InputRadioName,
        );
        let result = lease.advance(&cancel);
        let actual = source::formation::mutation_observation::finish();
        match result {
            Ok(crate::internal::source::DomSourceProgress::Complete(_)) => {
                assert!(
                    lease
                        .snapshot()
                        .node(key)
                        .unwrap()
                        .as_input()
                        .unwrap()
                        .state()
                        .flags()
                        .checkedness()
                );
                assert!(
                    !lease
                        .snapshot()
                        .node(peer)
                        .unwrap()
                        .as_input()
                        .unwrap()
                        .state()
                        .flags()
                        .checkedness()
                );
            }
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                ..
            }) => {
                assert_eq!(lease.snapshot().stamp(), before.stamp());
                assert!(
                    !lease
                        .snapshot()
                        .node(key)
                        .unwrap()
                        .as_input()
                        .unwrap()
                        .state()
                        .flags()
                        .checkedness()
                );
                assert!(
                    lease
                        .snapshot()
                        .node(peer)
                        .unwrap()
                        .as_input()
                        .unwrap()
                        .state()
                        .flags()
                        .checkedness()
                );
            }
            other => panic!("actual Radio normalization result: {other:?}"),
        }
        actual
    }
    #[test]
    fn source_radio_padded_name_alias_keeps_peer_comparison_and_rollback_admitted() {
        let high = source_radio_name(16_000_000).expect("actual shared Radio name");
        assert!(high.required <= high.limit);
        if let Some(exact) = source_radio_name(high.required) {
            assert!(exact.required <= exact.limit);
        }
        if let Some(short) = source_radio_name(high.required - 1) {
            assert!(short.required <= short.limit);
        }
    }
    #[test]
    fn reached_radio_peer_and_current_value_prefix_is_discarded_on_cancel_or_exhaustion() {
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
                    crate::internal::name_limits::DomNameLimits::new(
                        16_777_216, 16_384, 4_000_000, 1_048_576,
                    )
                    .unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let cancel = AtomicBool::new(false);
            let mut b = DomBatch::new();
            let document = b.create_document();
            let root =
                b.create_element_ns(&document, surgeist_html::values::Namespace::html(), "div");
            let first =
                b.create_element_ns(&document, surgeist_html::values::Namespace::html(), "input");
            let second =
                b.create_element_ns(&document, surgeist_html::values::Namespace::html(), "input");
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
            let DomNodeKind::Element(first_state) = &first_state.kind() else {
                unreachable!()
            };
            assert!(!first_state.input_state().unwrap().flags().checkedness());
            let second_node = prepared.state.node(dom.owner, second).unwrap();
            let DomNodeKind::Element(second_state) = &second_node.kind() else {
                unreachable!()
            };
            assert!(second_state.input_state().unwrap().flags().checkedness());
            assert_eq!(
                second_node.attribute_ids().iter().find_map(|id| {
                    let attr = prepared
                        .state
                        .node(dom.owner, *id)
                        .unwrap()
                        .attribute_node()
                        .unwrap()
                        .0;
                    (attr.name().local_name() == &DocumentString::from("value"))
                        .then_some(attr.value())
                }),
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
        use surgeist_html::elements::input::{
            HtmlInputEmptySelectionSupport as Empty,
            HtmlInputNoneDirectionSupport as NoneDirection,
            HtmlInputSelectionDirection as Direction,
        };
        for empty in [Empty::Supported, Empty::Unsupported] {
            for none in [NoneDirection::Supported, NoneDirection::Unsupported] {
                let mut dom = Dom::with_native_input_platform(
                    DomLimits::new(
                        16,
                        1024,
                        32,
                        0,
                        0,
                        4,
                        65536,
                        crate::internal::name_limits::DomNameLimits::new(
                            16_777_216, 16_384, 4_000_000, 1_048_576,
                        )
                        .unwrap(),
                    )
                    .unwrap(),
                    crate::internal::document::DomNativeInputPlatform::new(empty, none),
                )
                .unwrap();
                let cancel = AtomicBool::new(false);
                let mut b = DomBatch::new();
                let d = b.create_document();
                let input =
                    b.create_element_ns(&d, surgeist_html::values::Namespace::html(), "input");
                b.set_input_value(&input, "abcd");
                let c = dom.apply(b, &cancel).unwrap();
                let input = c.created(&input).unwrap().unwrap();
                let mut draft = Draft::new(&dom, Arc::new(BatchIdentity), &cancel).unwrap();
                let key = draft.resolve(input.into()).unwrap();
                // A valid established range fixture; public range/event operations
                // remain separate. Exercise the real native current-value path.
                draft
                    .with_input_state(key, &cancel, |state| {
                        state.selection =
                            HtmlInputSelection::selection(2, 4, Direction::Backward).unwrap()
                    })
                    .unwrap();
                draft
                    .set_input_value(key, &"ab".into(), dom.limits, &cancel)
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
