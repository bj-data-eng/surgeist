//! Native Input and Textarea selection over a single Source draft.

use super::*;
use crate::internal::{
    compound::TextSelectionEdit,
    textarea::{normalized_length, visit_normalized_units},
};
use surgeist_html::elements::input::{
    HtmlInputSelectMethod, HtmlInputSelection, HtmlRangeTextError, HtmlRangeTextPlan,
    HtmlRangeTextTarget, HtmlTextSelectionEdit,
};

#[derive(Clone, Copy)]
enum Control {
    Input,
    Textarea,
}

impl Draft {
    pub(super) fn edit_text_control_selection(
        &mut self,
        key: Key,
        edit: &TextSelectionEdit,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let language = self.element(key)?.language();
        let control = if let Some(input) = language.input_state() {
            let kind = input.input_type();
            if matches!(
                edit,
                TextSelectionEdit::Selection(HtmlTextSelectionEdit::All)
            ) {
                // The current native profile has value text controls. It
                // supplies no alternate picker or rendered control text.
                if kind.select_method() != HtmlInputSelectMethod::CurrentValue {
                    return Ok(());
                }
            } else if !kind.is_selectable() {
                return Err(DomError::InputSelectionInvalidState);
            }
            Control::Input
        } else if language.textarea_state().is_some() {
            Control::Textarea
        } else {
            return Err(DomError::WrongNodeKind);
        };

        match edit {
            TextSelectionEdit::Selection(edit) => {
                let len = self.text_selection_value_len(key, control, cancel)?;
                self.apply_text_selection(key, control, *edit, len, cancel)
            }
            TextSelectionEdit::RangeText {
                target,
                replacement,
            } => self.replace_text_selection(key, control, *target, replacement, limits, cancel),
        }
    }

    fn text_selection_value_len(
        &self,
        key: Key,
        control: Control,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        self.source_step(cancel)?;
        match control {
            Control::Input => Ok(self.input_state(key)?.current.len()),
            Control::Textarea => {
                normalized_length(self.textarea_state(key)?.raw.code_units(), || {
                    self.source_step(cancel)
                })
            }
        }
    }

    fn text_selection(&self, key: Key, control: Control) -> Result<HtmlInputSelection, DomError> {
        match control {
            Control::Input => Ok(self.input_state(key)?.selection),
            Control::Textarea => Ok(self.textarea_state(key)?.selection),
        }
    }

    fn apply_text_selection(
        &mut self,
        key: Key,
        control: Control,
        edit: HtmlTextSelectionEdit,
        len: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let platform = self
            .document_metadata(self.node(key)?.document())?
            .document_environment()
            .input_platform();
        let previous = self.text_selection(key, control)?;
        let next = previous.edit(
            edit,
            len,
            platform.empty_selection(),
            platform.none_direction(),
        );
        if previous == next {
            return Ok(());
        }
        match control {
            Control::Input => self.with_input_state(key, cancel, |state| state.selection = next)?,
            Control::Textarea => {
                self.with_textarea_state(key, cancel, |state| state.selection = next)?
            }
        }
        self.queue_text_selection_event(key, cancel)
    }

    fn replace_text_selection(
        &mut self,
        key: Key,
        control: Control,
        target: HtmlRangeTextTarget,
        replacement: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.prepare_compound_failure_slot(cancel)?;
        match control {
            Control::Input => {
                let flags = self
                    .input_state(key)?
                    .flags
                    .range_text_started()
                    .ok_or(DomError::InputSelectionInvalidState)?;
                self.with_input_state(key, cancel, |state| state.flags = flags)?;
            }
            Control::Textarea => self.with_textarea_state(key, cancel, |state| {
                state.flags.mark_dirty_value();
            })?,
        }
        self.source_step(cancel)?;
        let bounds = match target.resolve(self.text_selection(key, control)?) {
            Ok(bounds) => bounds,
            Err(HtmlRangeTextError::Reversed) => return self.compound_error(DomError::IndexSize),
            Err(HtmlRangeTextError::ExtentOverflow) => {
                return Err(DomError::RepresentationExhausted);
            }
        };
        let len = self.text_selection_value_len(key, control, cancel)?;
        let plan = bounds
            .plan(len, replacement.len())
            .map_err(|_| DomError::RepresentationExhausted)?;
        let (value, payload) =
            self.splice_text_selection(key, control, &plan, replacement, limits, cancel)?;
        let len = match control {
            Control::Input => {
                let len = value.len();
                self.input_put_value(
                    key,
                    input_processing::InputValue { value, payload },
                    None,
                    limits,
                    cancel,
                )?;
                len
            }
            Control::Textarea => self.textarea_put_raw(key, value, payload, limits, cancel)?,
        };
        // The actual value assignment has applied its relevant-value clamp.
        // Only the following explicit selection update can queue this event.
        self.apply_text_selection(key, control, plan.selection(), len, cancel)
    }

    fn splice_text_selection(
        &self,
        key: Key,
        control: Control,
        plan: &HtmlRangeTextPlan,
        replacement: &DocumentString,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(DocumentString, Option<source::checkpoint::Charge>), DomError> {
        let raw = match control {
            Control::Input => self.input_state(key)?.current.code_units(),
            Control::Textarea => self.textarea_state(key)?.raw.code_units(),
        };
        let len = plan.output_len();
        let total = (self.units - raw.len())
            .checked_add(len)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(total, limits.units, DomResource::StringCodeUnits)?;
        let formation = source::formation::Formation::new(self, cancel)?;
        let mut output =
            source::formation::utf16_buffer(len, limits.units, formation.as_ref(), cancel)?;
        let replaced = plan.replaced();
        match control {
            Control::Input => {
                let prefix = raw
                    .get(..replaced.start)
                    .ok_or(DomError::RepresentationExhausted)?;
                let tail = raw
                    .get(replaced.end..)
                    .ok_or(DomError::RepresentationExhausted)?;
                for &unit in prefix.iter().chain(replacement.code_units()).chain(tail) {
                    self.source_step(cancel)?;
                    output.push(unit);
                }
            }
            Control::Textarea => {
                let mut index = 0;
                let mut step = || self.source_step(cancel);
                visit_normalized_units(raw, &mut step, |unit, step| {
                    if index == replaced.start {
                        for &inserted in replacement.code_units() {
                            step()?;
                            output.push(inserted);
                        }
                    }
                    if !replaced.contains(&index) {
                        step()?;
                        output.push(unit);
                    }
                    index += 1;
                    Ok(())
                })?;
                if index == replaced.start {
                    for &unit in replacement.code_units() {
                        step()?;
                        output.push(unit);
                    }
                }
            }
        }
        Ok((
            DocumentString::from_utf16(output),
            formation.map(source::formation::Formation::into_charge),
        ))
    }
}
