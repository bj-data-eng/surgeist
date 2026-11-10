//! Input element value state: initialization, type changes, sanitization and reset.

use super::*;
use surgeist_html::{
    bounded::form::{
        HtmlFormOwnerResetDecision, HtmlFormOwnerResetTrigger, form_owner_reset_trigger,
    },
    construction::ConstructionNodeKind,
    values::{Attribute, Namespace, OrderedAttributes, QualifiedName},
};

/// Creation/mutation initializes from installed attributes. A Color clone
/// instead preserves its qualified original current value at clone completion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InputAttributeInitialization {
    Installed,
    ClonedColor,
}

impl SessionStorage {
    pub(super) fn input_raw_slot(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<usize, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::Input { raw, .. }) = self.payloads.values.get(slot) else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<usize>() as u64,
        )?;
        Ok(*raw)
    }
    pub(super) fn read_input_state(
        &self,
        control: &mut ParseControl,
        slot: usize,
    ) -> Result<InputState, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        control.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<InputState>() as u64,
        )?;
        let Some(RetainedPayload::Input { state, .. }) = self.payloads.values.get(slot) else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        Ok(*state)
    }
    pub(super) fn write_input_state(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        next: InputState,
    ) -> Result<(), SessionError> {
        payload_write::<InputState>(control, 1)?;
        let Some(RetainedPayload::Input { state, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(SessionError::InputMetadataUnavailable);
        };
        *state = next;
        Ok(())
    }
    pub(super) fn replace_input_raw(
        &mut self,
        control: &mut ParseControl,
        slot: usize,
        text: FieldCopy,
    ) -> Result<(), SessionError> {
        let previous = match self.input_raw_slot(control, slot) {
            Ok(previous) => previous,
            Err(error) => {
                text.release(control)?;
                return Err(error);
            }
        };
        // Both fresh initialization and clone initialization already run inside
        // the same qualified owner. A clone's retained construction frame is
        // legitimate useful work, so do not reenter the external idle gate.
        let dispose_previous = match self.releasable_string(control, previous, false) {
            Ok(value) => value,
            Err(error) => {
                text.release(control)?;
                return Err(error);
            }
        };
        let (value, next) = self.register_field_copy(control, text)?;
        drop(value); // no native alias: actual independent raw owner only
        payload_write::<usize>(control, 1)?;
        payload_write::<RetainedPayload>(control, 2)?;
        self.retire_field_group(control, previous)?;
        let Some(RetainedPayload::Input { raw, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(latch_registry_row(control));
        };
        *raw = next;
        if dispose_previous {
            self.dispose_aggregate_string(control, previous)?;
        }
        Ok(())
    }
    pub(super) fn attribute_value_slot(
        &self,
        control: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<usize, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let attributes = element.attributes();
        let value = attributes
            .get(position)
            .ok_or(SessionError::AttributeStorageUnavailable)?
            .value();
        // Valid-or-fallback hint: creation registered this element's attribute
        // names and values as adjacent registry rows, last attribute first. The
        // candidate row is used only after its identity matches the attribute
        // value; any mismatch, or a hint cleared by an attribute edit, falls
        // back to the scan below, which returns the same row.
        if let Some(candidate) =
            self.payloads
                .attribute_value_candidate(node, attributes.len(), position)
        {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::String { value: stored, .. }) =
                self.payloads.values.get(candidate)
            {
                control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                if value.shares_storage_with(stored) {
                    #[cfg(test)]
                    assert_eq!(
                        self.payloads.first_string_sharing(value),
                        Some(candidate),
                        "a verified hint names the row a scan finds"
                    );
                    return Ok(candidate);
                }
            }
        }
        for (slot, payload) in self.payloads.values.iter().enumerate() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::String {
                value: candidate, ..
            } = payload
            {
                control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                if value.shares_storage_with(candidate) {
                    return Ok(slot);
                }
            }
        }
        Err(SessionError::AttributeStorageUnavailable)
    }
    fn install_sanitized_input_value(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        source: usize,
        policy: surgeist_html::elements::input::HtmlInputValueSanitization,
    ) -> Result<(), SessionError> {
        let policy = match policy {
            surgeist_html::elements::input::HtmlInputValueSanitization::String(policy) => policy,
            surgeist_html::elements::input::HtmlInputValueSanitization::Range => {
                return self.install_range_input_value(control, node, slot, source);
            }
            surgeist_html::elements::input::HtmlInputValueSanitization::Color => {
                return self.update_input_color(control, node, slot);
            }
        };
        if matches!(
            policy,
            surgeist_html::elements::input::HtmlInputSanitization::Email
                | surgeist_html::elements::input::HtmlInputSanitization::StripCrLfAndTrim
                | surgeist_html::elements::input::HtmlInputSanitization::Temporal(_)
                | surgeist_html::elements::input::HtmlInputSanitization::Number
        ) {
            let mode = if policy == surgeist_html::elements::input::HtmlInputSanitization::Email
                && self.form_attribute(control, node, "multiple")?.is_some()
            {
                surgeist_html::elements::input::HtmlInputEmailMode::Multiple
            } else {
                surgeist_html::elements::input::HtmlInputEmailMode::Single
            };
            let (field, count) = copy_selected_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                source,
                policy,
                mode,
            )?;
            self.replace_input_raw(control, slot, field)?;
            let mut state = self.read_input_state(control, slot)?;
            state.email_value_count = count;
            self.write_input_state(control, slot, state)?;
        } else {
            let field = if policy == surgeist_html::elements::input::HtmlInputSanitization::Preserve
            {
                copy_preserved_input_value(&self.payloads, self.evidence.as_ref(), control, source)?
            } else {
                copy_input_value(
                    &self.payloads,
                    self.evidence.as_ref(),
                    control,
                    source,
                    policy,
                )?
            };
            self.replace_input_raw(control, slot, field)?;
        }
        Ok(())
    }
    pub(super) fn initial_input_attribute(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        association: usize,
        attribute: (usize, usize),
        initialization: InputAttributeInitialization,
    ) -> Result<(), SessionError> {
        let (container, position) = attribute;
        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        let named = element_attribute(&self.construction, control, node, position)?;
        let name = named.name().local_name();
        let value_attr = exact_ascii(control, Some(name), "value")?;
        let type_attr = exact_ascii(control, Some(name), "type")?;
        let checked_attr = exact_ascii(control, Some(name), "checked")?;
        let form_attr = exact_ascii(control, Some(name), "form")?;
        let name_attr =
            named.name().namespace().uri().is_none() && exact_ascii(control, Some(name), "name")?;
        let state = self.read_input_state(control, slot)?;
        let src_attr = state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Image
            && named.name().namespace().uri().is_none()
            && exact_ascii(control, Some(name), "src")?;
        let multiple_attr = state.flags.input_type()
            == surgeist_html::attributes::HtmlInputType::Email
            && exact_ascii(control, Some(name), "multiple")?;
        if value_attr
            && initialization != InputAttributeInitialization::ClonedColor
            && let surgeist_html::elements::input::HtmlInputValueAttributeEffect::ReplaceCurrentFromInstalledDefault(policy) =
                state.flags.value_attribute_changed()
        {
            let source = self.attribute_value_slot(control, node, position)?;
            self.install_sanitized_input_value(control, node, slot, source, policy)?;
        }
        if checked_attr
            && let Some(flags) = state.flags.checked_attribute_changed(
                surgeist_html::elements::input::HtmlInputCheckedAttributeChange::Added,
            )
        {
            let mut next = state;
            next.flags = flags;
            self.write_input_state(control, slot, next)?;
            if flags.radio_normalization_required(
                surgeist_html::elements::input::HtmlRadioTrigger::Checkedness,
            ) {
                self.normalize_radio(control, node, None)?;
            }
        }
        if type_attr {
            let attribute = element_attribute(&self.construction, control, node, position)?;
            let next_kind = input_type_state(control, attribute.value())?;
            self.change_input_type(control, node, slot, container, state, next_kind)?;
        }
        if src_attr {
            self.image_source_changed(control, node, slot)?;
        }
        if name_attr
            && state.flags.radio_normalization_required(
                surgeist_html::elements::input::HtmlRadioTrigger::Name,
            )
        {
            self.normalize_radio(control, node, None)?;
        }
        if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Email
            && multiple_attr
        {
            let raw = self.input_raw_slot(control, slot)?;
            self.install_sanitized_input_value(
                control,
                node,
                slot,
                raw,
                surgeist_html::elements::input::HtmlInputValueSanitization::String(
                    surgeist_html::elements::input::HtmlInputSanitization::Email,
                ),
            )?;
        }
        if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Range {
            let attribute = element_attribute(&self.construction, control, node, position)?;
            let relevant = attribute.name().namespace().uri().is_none()
                && (exact_ascii(control, Some(attribute.name().local_name()), "min")?
                    || exact_ascii(control, Some(attribute.name().local_name()), "max")?
                    || exact_ascii(control, Some(attribute.name().local_name()), "step")?);
            if relevant {
                let raw = self.input_raw_slot(control, slot)?;
                self.install_range_input_value(control, node, slot, raw)?;
            }
        }
        if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Color
            && initialization != InputAttributeInitialization::ClonedColor
        {
            let attribute = element_attribute(&self.construction, control, node, position)?;
            let relevant = attribute.name().namespace().uri().is_none()
                && (exact_ascii(control, Some(attribute.name().local_name()), "alpha")?
                    || exact_ascii(control, Some(attribute.name().local_name()), "colorspace")?);
            if relevant {
                self.update_input_color(control, node, slot)?;
            }
        }
        if form_attr {
            let state = find_form_association_state(&self.payloads, control, node)?
                .ok_or(SessionError::FormAssociationMetadataUnavailable)?
                .1;
            if form_owner_reset_trigger(HtmlFormOwnerResetTrigger::FormAttributeChanged {
                listed: state.kind.listed(),
            }) == HtmlFormOwnerResetDecision::Reset
            {
                self.reset_form_association(control, association, node, state, None)?;
            }
        }
        Ok(())
    }
    // The same actual state transition is consumed by ordered creation and
    // managed type removal/change. Storage and source effects stay here.
    pub(super) fn change_input_type(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        container: usize,
        state: InputState,
        next_kind: surgeist_html::attributes::HtmlInputType,
    ) -> Result<(), SessionError> {
        if next_kind != state.flags.input_type() {
            let mut next = state;
            next.flags = state.flags.type_changed(next_kind);
            if next_kind != surgeist_html::attributes::HtmlInputType::Email {
                next.email_value_count = None;
            }
            self.write_input_state(control, slot, next)?;
            let raw = self.input_raw_slot(control, slot)?;
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let current_len = textarea_raw(&self.payloads, control, raw)?.len();
            let mut change = surgeist_html::elements::input::HtmlInputTypeChange::new(
                state.flags.input_type(),
                next_kind,
                current_len,
            );
            while let Some(effect) = change.next_effect() {
                if effect == surgeist_html::elements::input::HtmlInputTypeEffect::SetContentValueFromCurrent {
                    let mut value_position = None;
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let element = element_record(&self.construction, control, node)?;
                    for (index, attribute) in element.attributes().iter().enumerate() {
                        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        if exact_ascii(control, Some(attribute.name().local_name()), "value")? {
                            value_position = Some(index);
                            break;
                        }
                    }
                    let generated = matches!(
                        state.flags.input_type(),
                        surgeist_html::attributes::HtmlInputType::Range | surgeist_html::attributes::HtmlInputType::Color
                    );
                    let position = match value_position {
                        Some(position) => {
                            self.writeback_input_value(
                                control, node, slot, container, position, raw,
                            )?;
                            position
                        }
                        None => self.append_input_value_from_current(
                            control, node, slot, container, raw, generated,
                        )?,
                    };
                    // Nested dirty=false value change in Default or DefaultOn:
                    // copy the actual replaced native field without a sanitizer.
                    let source = self.attribute_value_slot(control, node, position)?;
                    let text = if generated {
                        provenance::copy_generated_input_value(
                            &self.payloads,
                            self.evidence.as_ref(),
                            control,
                            source,
                        )?
                    } else {
                        copy_input_value(
                            &self.payloads,
                            self.evidence.as_ref(),
                            control,
                            source,
                            surgeist_html::elements::input::HtmlInputSanitization::Preserve,
                        )?
                    };
                    self.replace_input_raw(control, slot, text)?;
                }
                if effect == surgeist_html::elements::input::HtmlInputTypeEffect::SetCurrentFromDefaultAndClearDirty {
                    let mut source = None;
                    control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    let element = element_record(&self.construction, control, node)?;
                    for (position, attribute) in element.attributes().iter().enumerate()
                    {
                        control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                        if attribute.name().namespace().uri().is_none()
                            && exact_ascii(control, Some(attribute.name().local_name()), "value")?
                        {
                            source = Some(self.attribute_value_slot(control, node, position)?);
                            break;
                        }
                    }
                    let field = match source {
                        Some(source) => copy_preserved_input_value(
                            &self.payloads,
                            self.evidence.as_ref(),
                            control,
                            source,
                        )?,
                        None => FieldCopy {
                            value: literal_string(control, b"")?,
                            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                            original: None,
                        },
                    };
                    self.replace_input_raw(control, slot, field)?;
                    let mut current = self.read_input_state(control, slot)?;
                    current.flags = current.flags.type_default_installed();
                    self.write_input_state(control, slot, current)?;
                }
                if effect == surgeist_html::elements::input::HtmlInputTypeEffect::SetCurrentEmptyForFilename {
                    let field = FieldCopy {
                        value: literal_string(control, b"")?,
                        origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                        original: None,
                    };
                    self.replace_input_raw(control, slot, field)?;
                }
                if let surgeist_html::elements::input::HtmlInputTypeEffect::Sanitize(policy) =
                    effect
                    && policy
                        != surgeist_html::elements::input::HtmlInputValueSanitization::String(
                            surgeist_html::elements::input::HtmlInputSanitization::Preserve,
                        )
                {
                    let raw = self.input_raw_slot(control, slot)?;
                    self.install_sanitized_input_value(control, node, slot, raw, policy)?;
                }
                if effect == surgeist_html::elements::input::HtmlInputTypeEffect::SignalTypeChange
                    && next_kind == surgeist_html::attributes::HtmlInputType::Radio
                {
                    self.normalize_radio(control, node, None)?;
                }
                // Rendering has no host recipient in
                // fresh immutable construction. Initial cursor is already0.
            }
            if next_kind == surgeist_html::attributes::HtmlInputType::Image {
                self.image_type_entered(control, node, slot)?;
            }
        }
        Ok(())
    }
    fn append_input_value_from_current(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        container: usize,
        raw: usize,
        generated: bool,
    ) -> Result<usize, SessionError> {
        // "value" is a source-required generated name, not a tokenizer field.
        // The current cause is the actual type transition; no lexical atom is
        // invented. The value separately retains its real prior group.
        let name = FieldCopy {
            value: literal_string(control, b"value")?,
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
            original: None,
        };
        let (name, name_slot) = self.register_field_copy(control, name)?;
        let value = if generated {
            provenance::copy_generated_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                raw,
            )?
        } else {
            copy_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                control,
                raw,
                surgeist_html::elements::input::HtmlInputSanitization::Preserve,
            )?
        };
        let (value, value_slot) = self.register_field_copy(control, value)?;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(control))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let position = element.attributes().len();
        let mut state = self.read_input_state(control, slot)?;
        state.value_rewritten_during_creation = true;
        payload_write::<InputState>(control, 1)?;
        payload_write::<Attribute>(control, 1)?;
        let incoming = Attribute::new(QualifiedName::new(Namespace::none(), None, name), value);
        self.append_registered_attribute(
            control,
            node,
            container,
            incoming,
            [name_slot, value_slot],
        )?;
        let Some(RetainedPayload::Input {
            state: destination, ..
        }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        *destination = state;
        Ok(position)
    }
    fn writeback_input_value(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
        container: usize,
        position: usize,
        raw: usize,
    ) -> Result<(), SessionError> {
        let old_slot = self.attribute_value_slot(control, node, position)?;
        let text =
            copy_preserved_input_value(&self.payloads, self.evidence.as_ref(), control, raw)?;
        let (value, new_slot) = self.register_field_copy(control, text)?;
        (|| {
            control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
            payload_write::<QualifiedName>(control, 1)?;
            let attribute = element_attribute(&self.construction, control, node, position)?;
            payload_write::<Attribute>(control, 1)?;
            payload_write::<Option<Attribute>>(control, 1)?;
            let mut incoming = Some(Attribute::new(attribute.name().clone(), value));
            // Registry redirects and known shared-wrapper restoration are
            // preadmitted. The actual slot producer then commits all or none.
            payload_write::<[StringLocation; 2]>(control, 2)?;
            let mut next_state = self.read_input_state(control, slot)?;
            next_state.value_rewritten_during_creation = true;
            payload_write::<InputState>(control, 1)?;
            payload_write::<[Option<AttributeStorage>; 2]>(control, 2)?;
            control.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
            payload_write::<OrderedAttributes>(control, 1)?;
            let result = self.replace_attribute_storage(
                control,
                node,
                container,
                position,
                &mut incoming,
                &[Some(old_slot)],
            );
            match result {
                Ok(old) => {
                    self.bind_string(old_slot, StringLocation::Detached)
                        .map_err(|_| latch_registry_row(control))?;
                    self.bind_string(
                        new_slot,
                        StringLocation::AttributeValue(container, position),
                    )
                    .map_err(|_| latch_registry_row(control))?;
                    let Some(RetainedPayload::Input { state, .. }) =
                        self.payloads.values.get_mut(slot)
                    else {
                        return Err(latch_registry_row(control));
                    };
                    *state = next_state;
                    drop(old);
                    Ok(())
                }
                Err(stop) => {
                    drop(incoming);
                    Err(stop)
                }
            }
        })()
    }
    pub(super) fn reset_input(
        &mut self,
        control: &mut ParseControl,
        node: usize,
        slot: usize,
    ) -> Result<(), SessionError> {
        let mut state = self.read_input_state(control, slot)?;
        // Only fresh construction/clone initialization reaches this adapter.
        // Checked presence is already installed, files are already empty and
        // no partial input is observable before complete=true. Default copy
        // and sanitization may therefore fuse; generic reset remains ordered.
        state.flags = state.flags.reset_begin();
        self.write_input_state(control, slot, state)?;
        let mut checked = false;
        let mut value = None;
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let element = element_record(&self.construction, control, node)?;
        for (position, attribute) in element.attributes().iter().enumerate() {
            control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if exact_ascii(control, Some(attribute.name().local_name()), "checked")? {
                checked = true;
            }
            if exact_ascii(control, Some(attribute.name().local_name()), "value")? {
                value = Some(position);
            }
        }
        if let Some(position) = value {
            let source = self.attribute_value_slot(control, node, position)?;
            self.install_sanitized_input_value(
                control,
                node,
                slot,
                source,
                state.flags.input_type().sanitization(),
            )?;
        } else {
            let field = FieldCopy {
                value: literal_string(control, b"")?,
                origins: OwnedVec::new(HtmlParseResource::SourceMapEntries)?,
                original: None,
            };
            self.replace_input_raw(control, slot, field)?;
            if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Range {
                let raw = self.input_raw_slot(control, slot)?;
                self.install_range_input_value(control, node, slot, raw)?;
            }
            if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Color {
                self.update_input_color(control, node, slot)?;
            }
            if state.flags.input_type() == surgeist_html::attributes::HtmlInputType::Email {
                state.email_value_count =
                    if self.form_attribute(control, node, "multiple")?.is_some() {
                        Some(0)
                    } else {
                        None
                    };
                self.write_input_state(control, slot, state)?;
            }
        }
        if matches!(
            state.flags.input_type(),
            surgeist_html::attributes::HtmlInputType::Email
                | surgeist_html::attributes::HtmlInputType::Url
        ) {
            state = self.read_input_state(control, slot)?;
        }
        state.flags = state.flags.reset_checkedness(checked);
        self.write_input_state(control, slot, state)?;
        if state.flags.radio_normalization_required(
            surgeist_html::elements::input::HtmlRadioTrigger::Checkedness,
        ) {
            self.normalize_radio(control, node, None)?;
        }
        state.selected_files_empty = true;
        self.write_input_state(control, slot, state)?;
        payload_write::<bool>(control, 1)?;
        let Some(RetainedPayload::Input { complete, .. }) = self.payloads.values.get_mut(slot)
        else {
            return Err(latch_registry_row(control));
        };
        *complete = true;
        #[cfg(test)]
        if let Some(cancellation) = self.cancel_after_input_reset.take() {
            cancellation.cancel();
        }
        Ok(())
    }
}
