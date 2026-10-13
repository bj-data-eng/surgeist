//! Actual managed lexical field replacement/removal over the existing registry.
//! Family policies remain source-specific; this owner is the single backing edit.
use super::*;

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum ManagedFamily {
    Details,
    Input,
}
#[cfg(test)]
impl ManagedFamily {
    fn local(self) -> &'static str {
        match self {
            Self::Details => "details",
            Self::Input => "input",
        }
    }
    fn boolean(self) -> &'static str {
        match self {
            Self::Details => "open",
            Self::Input => "checked",
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum ManagedEffect {
    Details(crate::HtmlDetailsAttributeEffect),
    InputName,
    InputType,
    InputSource,
    InputColor,
    InputValue,
    InputChecked(Option<crate::HtmlInputCheckedAttributeChange>),
}
impl SessionStorage {
    #[cfg(test)]
    fn managed_effect(
        &self,
        c: &mut ParseControl,
        family: ManagedFamily,
        local: &str,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
    ) -> Result<ManagedEffect, SessionError> {
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<ManagedEffect>(c, 1)?;
        Ok(match family {
            ManagedFamily::Details => ManagedEffect::Details(crate::details_attribute_effect(
                &Namespace::none(),
                if local == "name" {
                    crate::HtmlDetailsAttribute::Name
                } else {
                    crate::HtmlDetailsAttribute::Open
                },
                old,
                new,
            )),
            ManagedFamily::Input if local == "type" => ManagedEffect::InputType,
            ManagedFamily::Input if local == "src" => ManagedEffect::InputSource,
            ManagedFamily::Input if local == "name" => ManagedEffect::InputName,
            ManagedFamily::Input if matches!(local, "alpha" | "colorspace") => {
                ManagedEffect::InputColor
            }
            ManagedFamily::Input if local == "value" => ManagedEffect::InputValue,
            ManagedFamily::Input => {
                ManagedEffect::InputChecked(match (old.is_some(), new.is_some()) {
                    (false, true) => Some(crate::HtmlInputCheckedAttributeChange::Added),
                    (true, false) => Some(crate::HtmlInputCheckedAttributeChange::Removed),
                    _ => None,
                })
            }
        })
    }
    #[cfg(test)]
    fn preflight_managed_effect(
        &self,
        c: &mut ParseControl,
        node: usize,
        effect: ManagedEffect,
    ) -> Result<(), SessionError> {
        if matches!(
            effect,
            ManagedEffect::Details(crate::HtmlDetailsAttributeEffect::Notify { .. })
        ) {
            self.require_details_notification(c, node)?;
        }
        Ok(())
    }
    #[cfg(test)]
    fn managed_change(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        effect: ManagedEffect,
    ) -> Result<(), SessionError> {
        match effect {
            ManagedEffect::Details(effect) => self.details_change(c, node, effect),
            ManagedEffect::InputName => self.normalize_radio(c, node, None),
            ManagedEffect::InputType => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                let (association, _) = find_form_association_state(&self.payloads, c, node)?
                    .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                let container = self.element_attribute_container(c, node)?;
                let HtmlNodeKind::Element(element) = &self.records[node].kind else {
                    unreachable!()
                };
                let mut position = None;
                for (i, a) in element.attributes().iter().enumerate() {
                    c.charge(WorkCategory::RecordVisit, 2)?;
                    if a.name().namespace().uri().is_none()
                        && exact_ascii(c, Some(a.name().local_name()), "type")?
                    {
                        position = Some(i);
                        break;
                    }
                }
                match position {
                    Some(position) => self.initial_input_attribute(
                        c,
                        node,
                        slot,
                        association,
                        (container, position),
                        InputAttributeInitialization::Installed,
                    ),
                    None => self.change_input_type(
                        c,
                        node,
                        slot,
                        container,
                        state,
                        crate::HtmlInputType::Text,
                    ),
                }
            }
            ManagedEffect::InputColor | ManagedEffect::InputValue => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                if state.flags.input_type() == crate::HtmlInputType::Color
                    && (!matches!(effect, ManagedEffect::InputValue) || !state.flags.dirty_value())
                {
                    self.update_input_color(c, node, slot)?;
                }
                Ok(())
            }
            ManagedEffect::InputSource => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                if state.flags.input_type() == crate::HtmlInputType::Image {
                    self.image_source_changed(c, node, slot)?;
                }
                Ok(())
            }
            ManagedEffect::InputChecked(None) => Ok(()),
            ManagedEffect::InputChecked(Some(change)) => {
                let (slot, mut state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                let Some(flags) = state.flags.checked_attribute_changed(change) else {
                    return Ok(());
                };
                state.flags = flags;
                self.write_input_state(c, slot, state)?;
                self.normalize_radio(c, node, None)
            }
        }
    }

    // Actual one-attribute stage shared by lexical and generated writeback.
    // Preserve the original admission, transfer and native commit order.
    pub(super) fn append_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        container: usize,
        incoming: Attribute,
        fields: [usize; 2],
        units: usize,
    ) -> Result<(), SessionError> {
        let [name_slot, value_slot] = fields;
        let (values, allocation) = c.new_vector()?;
        if let Err(error) = self.prepare_payload_slot(c) {
            release_vector(c, values, allocation)?;
            drop(incoming);
            return Err(error);
        }
        let stage = self.payloads.values.len();
        self.payloads.values.push(RetainedPayload::Attributes(Some(
            AttributeStorage::Building {
                values,
                allocation,
                original_len: 1,
                reversed_pairs: 0,
            },
        )));
        let prepared = (|| {
            let RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                allocation,
                ..
            })) = &mut self.payloads.values[stage]
            else {
                unreachable!()
            };
            c.reserve_vector(values, allocation, 1)?;
            payload_write::<Attribute>(c, 1)?;
            payload_write::<[StringLocation; 2]>(c, 2)?;
            c.acquire(HtmlParseResource::ManagedAttributes, 1)?;
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = prepared {
            drop(incoming);
            return Err(error);
        }
        let RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. })) =
            &mut self.payloads.values[stage]
        else {
            unreachable!()
        };
        values.push(incoming);
        self.bind_string(name_slot, StringLocation::AttributeName(stage, 0));
        self.bind_string(value_slot, StringLocation::AttributeValue(stage, 0));
        self.edit_attributes(c, node, container, stage, 1, false)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(record) = &self.records[node].kind else {
            unreachable!()
        };
        let start = record.attributes().len();
        self.begin_attribute_transfer(c, stage, container, start)?;
        self.finish_attribute_merge(
            c,
            PreparedAttributeMerge {
                target: node,
                container,
                stage,
                start,
                count: 1,
                units,
            },
        )?;
        Ok(())
    }
    pub(super) fn replace_registered_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        location: [usize; 2],
        incoming: Attribute,
        fields: [usize; 2],
        unit_counts: [usize; 2],
    ) -> Result<(), SessionError> {
        let [container, position] = location;
        let [name_slot, value_slot] = fields;
        let [units, old_units] = unit_counts;
        payload_write::<Option<Attribute>>(c, 1)?;
        let mut incoming = Some(incoming);
        let replace = (|| {
            // Both old original groups retire before the admitted native
            // swap. Every error drops only this local alias; the actual
            // registered backing/tickets remain reachable in the owner.
            for slot in 0..self.payloads.values.len() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if matches!(&self.payloads.values[slot], RetainedPayload::String { location: StringLocation::AttributeName(o,p) | StringLocation::AttributeValue(o,p) | StringLocation::AttributeNamespace(o,p) | StringLocation::AttributePrefix(o,p), .. } if *o == container && *p == position)
                {
                    self.retire_field_group(c, slot)?;
                }
            }
            // All old/new fixed locations and usage are admitted before
            // the slot swap, so no fallible rebind follows the commit.
            payload_write::<[StringLocation; 6]>(c, 6)?;
            payload_write::<usize>(c, 1)?;
            let Some(updated_units) = self
                .usage
                .utf16_units
                .checked_sub(old_units)
                .and_then(|old| old.checked_add(units))
            else {
                return c
                    .representation(HtmlParseResource::BufferedUtf16Units)
                    .map_err(Into::into);
            };
            let mut old_slots = [None; 4];
            for (slot, payload) in self.payloads.values.iter().enumerate() {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if let RetainedPayload::String { location, .. } = payload {
                    let which = match *location {
                        StringLocation::AttributeName(o, p) if o == container && p == position => {
                            Some(0)
                        }
                        StringLocation::AttributeValue(o, p) if o == container && p == position => {
                            Some(1)
                        }
                        StringLocation::AttributeNamespace(o, p)
                            if o == container && p == position =>
                        {
                            Some(2)
                        }
                        StringLocation::AttributePrefix(o, p)
                            if o == container && p == position =>
                        {
                            Some(3)
                        }
                        _ => None,
                    };
                    if let Some(which) = which {
                        payload_write::<Option<usize>>(c, 1)?;
                        old_slots[which] = Some(slot);
                    }
                }
            }
            payload_write::<Option<AttributeStorage>>(c, 2)?;
            payload_write::<OrderedAttributes>(c, 1)?;
            c.charge(WorkCategory::ProviderCall, 2)?;
            let RetainedPayload::Attributes(backing) = &mut self.payloads.values[container] else {
                unreachable!()
            };
            let AttributeStorage::Frozen {
                values,
                allocation,
                mut shared,
                element,
            } = backing.take().expect("qualified attributes")
            else {
                unreachable!()
            };
            drop(values);
            let HtmlNodeKind::Element(record) = &mut self.records[node].kind else {
                unreachable!()
            };
            let result = c.replace_attribute_slot(
                record.attributes.parser_backing_mut(),
                &mut shared,
                &allocation,
                position,
                &mut incoming,
            );
            *backing = Some(AttributeStorage::Frozen {
                values: record.attributes.clone(),
                allocation,
                shared,
                element,
            });
            let old = result?;
            for slot in old_slots.into_iter().flatten() {
                self.bind_string(slot, StringLocation::Detached);
            }
            self.bind_string(
                name_slot,
                StringLocation::AttributeName(container, position),
            );
            self.bind_string(
                value_slot,
                StringLocation::AttributeValue(container, position),
            );
            self.usage.utf16_units = updated_units;
            drop(old);
            Ok::<_, SessionError>(())
        })();
        drop(incoming);
        replace?;
        Ok(())
    }
    pub(super) fn remove_managed_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
        self.payloads.check_aliases(&self.records, c)?;
        let container = self.element_attribute_container(c, node)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self.records[node].kind else {
            return Err(SessionError::WrongToken);
        };
        let mut selected = None;
        let mut units = 0;
        for (index, attr) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if attr.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attr.name().local_name()), local)?
            {
                selected = Some(index);
                units = attr.name().local_name().len()
                    + attr.value().len()
                    + attr.name().prefix().map_or(0, DocumentString::len);
                break;
            }
        }
        let Some(index) = selected else {
            return Ok(());
        };
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
            unreachable!()
        };
        let AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        } = storage.take().expect("qualified real attributes")
        else {
            unreachable!()
        };
        let len = values.len();
        *storage = Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            index,
            shifted: 0,
            popped: false,
            rebound: 0,
        });
        // Historical destinations stay at the original slot. Retire their
        // original groups before any location is rebound, preserving the cause.
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(&self.payloads.values[slot], RetainedPayload::String { location: StringLocation::AttributeName(owner, position) | StringLocation::AttributeValue(owner, position) | StringLocation::AttributeNamespace(owner, position) | StringLocation::AttributePrefix(owner, position), .. } if *owner == container && *position == index)
            {
                self.retire_field_group(c, slot)?;
            }
        }
        for position in index..len - 1 {
            self.edit_removing_attribute(c, node, container, Some(position))?;
        }
        self.edit_removing_attribute(c, node, container, None)?;
        for slot in 0..self.payloads.values.len() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            payload_write::<StringLocation>(c, 1)?;
            payload_write::<usize>(c, 1)?;
            if let RetainedPayload::String { location, .. } = &mut self.payloads.values[slot] {
                let (owner, position) = match *location {
                    StringLocation::AttributeName(o, p)
                    | StringLocation::AttributeValue(o, p)
                    | StringLocation::AttributeNamespace(o, p)
                    | StringLocation::AttributePrefix(o, p) => (o, p),
                    _ => (usize::MAX, 0),
                };
                if owner == container {
                    *location = if position == index {
                        StringLocation::Detached
                    } else {
                        let p = position - usize::from(position > index);
                        match *location {
                            StringLocation::AttributeName(..) => {
                                StringLocation::AttributeName(owner, p)
                            }
                            StringLocation::AttributeValue(..) => {
                                StringLocation::AttributeValue(owner, p)
                            }
                            StringLocation::AttributeNamespace(..) => {
                                StringLocation::AttributeNamespace(owner, p)
                            }
                            StringLocation::AttributePrefix(..) => {
                                StringLocation::AttributePrefix(owner, p)
                            }
                            _ => unreachable!(),
                        }
                    };
                }
            }
            let RetainedPayload::Attributes(Some(AttributeStorage::Removing { rebound, .. })) =
                &mut self.payloads.values[container]
            else {
                unreachable!()
            };
            *rebound = slot + 1;
        }
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        payload_write::<usize>(c, 2)?;
        let RetainedPayload::Attributes(storage) = &mut self.payloads.values[container] else {
            unreachable!()
        };
        let AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            ..
        } = storage.take().expect("retained removal")
        else {
            unreachable!()
        };
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        self.usage.attributes -= 1;
        self.usage.utf16_units -= units;
        Ok(())
    }
}

#[cfg(test)]
impl ParserArenaSession {
    pub(super) fn set_managed_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        input: &mut Option<crate::parser::tokenizer::token::Attribute>,
        capability: &crate::parser::tree_builder::ConstructionCapability<'_>,
        family: ManagedFamily,
    ) -> Result<(), SessionError> {
        capability.validate_session(c, self)?;
        let storage = self.active_mut()?;
        let node = storage.index(c, node)?;
        if !storage.html_local(c, node, family.local())? {
            return Err(SessionError::WrongToken);
        }
        storage.payloads.check_aliases(&storage.records, c)?;
        let attr = input.as_ref().ok_or(SessionError::Consumed)?;
        c.require_source_range(&attr.source)?;
        check_formatting_text(c, &attr.name, &attr.source)?;
        check_formatting_text(c, &attr.value, &attr.source)?;
        let local = if exact_ascii(c, Some(attr.name.value.text()), "name")? {
            "name"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "type")?
        {
            "type"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "src")?
        {
            "src"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "alpha")?
        {
            "alpha"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "colorspace")?
        {
            "colorspace"
        } else if matches!(family, ManagedFamily::Input)
            && exact_ascii(c, Some(attr.name.value.text()), "value")?
        {
            "value"
        } else if exact_ascii(c, Some(attr.name.value.text()), family.boolean())? {
            family.boolean()
        } else {
            return Err(SessionError::WrongToken);
        };
        if matches!(family, ManagedFamily::Input) {
            let (_, state) = find_input_state(&storage.payloads, c, node)?;
            if local == "type" {
                let next = input_type_state(c, attr.value.value.text())?;
                let file_image = matches!(
                    state.flags.input_type(),
                    crate::HtmlInputType::File | crate::HtmlInputType::Image
                ) || matches!(
                    next,
                    crate::HtmlInputType::File | crate::HtmlInputType::Image
                );
                let radio = next == crate::HtmlInputType::Radio
                    && matches!(
                        state.flags.input_type(),
                        crate::HtmlInputType::Radio | crate::HtmlInputType::Checkbox
                    );
                let color = state.flags.input_type() == crate::HtmlInputType::Color
                    || next == crate::HtmlInputType::Color;
                if !(radio || file_image || color) {
                    return Err(SessionError::WrongToken);
                }
            } else if matches!(local, "value" | "alpha" | "colorspace")
                && state.flags.input_type() != crate::HtmlInputType::Color
            {
                return Err(SessionError::WrongToken);
            }
        }
        let container = storage.element_attribute_container(c, node)?;
        let mut position = None;
        let mut old_units = 0;
        let HtmlNodeKind::Element(element) = &storage.records[node].kind else {
            unreachable!()
        };
        for (index, old) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if old.name().namespace().uri().is_none()
                && exact_ascii(c, Some(old.name().local_name()), local)?
            {
                position = Some(index);
                old_units = old.name().local_name().len() + old.value().len();
                break;
            }
        }
        let effect = storage.managed_effect(
            c,
            family,
            local,
            position.map(|p| element.attributes().as_slice()[p].value()),
            Some(attr.value.value.text()),
        )?;
        storage.preflight_managed_effect(c, node, effect)?;
        let previous = storage.begin_attribute_change(c, node, Some(&attr.source))?;
        payload_write::<Option<crate::parser::tokenizer::token::Attribute>>(c, 1)?;
        let attr = input.take().expect("qualified original owner");
        let result = (|| {
            let (name, name_slot) = match storage.register_text(c, attr.name) {
                Ok(value) => value,
                Err(error) => {
                    attr.value.release(c)?;
                    return Err(error);
                }
            };
            let (value, value_slot) = storage.register_text(c, attr.value)?;
            c.charge(WorkCategory::RecordVisit, 2)?;
            let units = name.len() + value.len();
            payload_write::<Attribute>(c, 1)?;
            let incoming = Attribute::new(QualifiedName::new(Namespace::none(), None, name), value);
            if let Some(position) = position {
                storage.replace_registered_attribute(
                    c,
                    node,
                    [container, position],
                    incoming,
                    [name_slot, value_slot],
                    [units, old_units],
                )?;
            } else {
                storage.append_registered_attribute(
                    c,
                    node,
                    container,
                    incoming,
                    [name_slot, value_slot],
                    units,
                )?;
            }
            storage.managed_change(c, node, effect)?;
            storage.drain_construction_effects(c)?;
            Ok(())
        })();
        if c.stop().is_none() {
            storage.end_node_copy(c, previous.as_ref())?;
        }
        match result {
            Ok(()) => Ok(()),
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) => Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into()),
        }
    }
    pub(super) fn remove_managed_attribute(
        &mut self,
        c: &mut ParseControl,
        node: &ParserNode,
        local: &str,
        family: ManagedFamily,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        let node = storage.index(c, node)?;
        if !storage.html_local(c, node, family.local())? {
            return Err(SessionError::WrongToken);
        }
        if matches!(family, ManagedFamily::Input) {
            find_input_state(&storage.payloads, c, node)?;
        }
        let Some(old) = storage.form_attribute(c, node, local)? else {
            return Ok(());
        };
        let effect = storage.managed_effect(c, family, local, Some(old), None)?;
        storage.preflight_managed_effect(c, node, effect)?;
        let previous = storage.begin_attribute_change(c, node, None)?;
        let result = (|| {
            storage.remove_managed_field(c, node, local)?;
            // Removing name still invokes the name-change algorithm; its
            // actual now-absent name provides the source early return.
            storage.managed_change(c, node, effect)?;
            storage.drain_construction_effects(c)?;
            Ok(())
        })();
        if c.stop().is_none() {
            storage.end_node_copy(c, previous.as_ref())?;
        }
        match result {
            Ok(()) => Ok(()),
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            Err(_) => Err(c
                .fail_invariant(crate::parser::control::Invariant::PendingFormEffects)
                .into()),
        }
    }
}
