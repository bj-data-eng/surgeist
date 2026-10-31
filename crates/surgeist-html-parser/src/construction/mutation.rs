//! Actual managed lexical field replacement/removal over the existing registry.
//! Family policies remain source-specific; this owner is the single backing edit.
use super::*;

#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum ManagedFamily {
    Base,
    Link,
    Details,
    Dialog,
    Input,
    Style,
}
#[cfg(test)]
impl ManagedFamily {
    fn local(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Link => "link",
            Self::Details => "details",
            Self::Dialog => "dialog",
            Self::Input => "input",
            Self::Style => "style",
        }
    }
    fn boolean(self) -> &'static str {
        match self {
            Self::Base => "target",
            Self::Link => "disabled",
            Self::Details | Self::Dialog => "open",
            Self::Input => "checked",
            Self::Style => "title",
        }
    }
}
#[cfg(test)]
#[derive(Clone, Copy)]
enum ManagedEffect {
    Dialog,
    Style,
    BaseHref,
    BaseTarget,
    LinkFlags(Option<crate::HtmlLinkFlags>),
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
        node: usize,
        local: &str,
        old: Option<&DocumentString>,
        new: Option<&DocumentString>,
    ) -> Result<ManagedEffect, SessionError> {
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<ManagedEffect>(c, 1)?;
        Ok(match family {
            ManagedFamily::Dialog => ManagedEffect::Dialog,
            ManagedFamily::Style => ManagedEffect::Style,
            ManagedFamily::Base if local == "href" => ManagedEffect::BaseHref,
            ManagedFamily::Base => ManagedEffect::BaseTarget,
            ManagedFamily::Link => {
                let flags = if local == "disabled" && old.is_some() && new.is_none() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let view = self
                        .construction
                        .node_at(node)
                        .ok_or(SessionError::ForeignNode)?;
                    let HtmlNodeKind::Element(element) = view.kind() else {
                        return Err(SessionError::WrongToken);
                    };
                    let mut flags = element.link_flags().ok_or(SessionError::WrongToken)?;
                    flags.disabled_attribute_changed(&Namespace::none(), old, new);
                    Some(flags)
                } else {
                    None
                };
                ManagedEffect::LinkFlags(flags)
            }
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
            // The generic actual backing owner already ran the exact Dialog
            // hook, including equal writes and removed attributes.
            ManagedEffect::Dialog | ManagedEffect::Style => Ok(()),
            ManagedEffect::BaseHref => self.base_change(c, node, None, true),
            ManagedEffect::BaseTarget => Ok(()),
            ManagedEffect::LinkFlags(flags) => {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if let Some(flags) = flags {
                    payload_write::<crate::HtmlLinkFlags>(c, 1)?;
                    let view = self
                        .construction
                        .node_at(node)
                        .ok_or(SessionError::ForeignNode)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    let element = view.handle();
                    self.construction.apply(
                        c,
                        bounded::HtmlConstructionCommand::SetLinkFlags { element, flags },
                        &[],
                    )?;
                }
                Ok(())
            }
            ManagedEffect::Details(effect) => self.details_change(c, node, effect),
            ManagedEffect::InputName => self.normalize_radio(c, node, None),
            ManagedEffect::InputType => {
                let (slot, state) = self
                    .radio_input(c, node)?
                    .ok_or(SessionError::InputMetadataUnavailable)?;
                let (association, _) = find_form_association_state(&self.payloads, c, node)?
                    .ok_or(SessionError::FormAssociationMetadataUnavailable)?;
                let container = self.element_attribute_container(c, node)?;
                let HtmlNodeKind::Element(element) = &self
                    .construction
                    .node_at(node)
                    .ok_or_else(|| latch_registry_row(c))?
                    .kind()
                else {
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
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building {
                values,
                allocation,
                ..
            }))) = self.payloads.values.get_mut(stage)
            else {
                return Err(latch_registry_row(c));
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
        let Some(RetainedPayload::Attributes(Some(AttributeStorage::Building { values, .. }))) =
            self.payloads.values.get_mut(stage)
        else {
            return Err(latch_registry_row(c));
        };
        values.push(incoming);
        self.bind_string(name_slot, StringLocation::AttributeName(stage, 0))?;
        self.bind_string(value_slot, StringLocation::AttributeValue(stage, 0))?;
        self.edit_attributes(c, node, container, stage, 1, false)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(record) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
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
    ) -> Result<(), SessionError> {
        let [container, position] = location;
        let [name_slot, value_slot] = fields;
        payload_write::<Option<Attribute>>(c, 1)?;
        let mut incoming = Some(incoming);
        let replace = (|| {
            // All fixed registry progress is admitted before the generic ready
            // boundary; old source history retires while that owner is exclusive.
            payload_write::<[StringLocation; 6]>(c, 6)?;
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
            let old = self.replace_attribute_storage(
                c,
                node,
                container,
                position,
                &mut incoming,
                &old_slots,
            )?;
            for slot in old_slots.into_iter().flatten() {
                self.bind_string(slot, StringLocation::Detached)?;
            }
            self.bind_string(
                name_slot,
                StringLocation::AttributeName(container, position),
            )?;
            self.bind_string(
                value_slot,
                StringLocation::AttributeValue(container, position),
            )?;

            drop(old);
            Ok::<_, SessionError>(())
        })();
        drop(incoming);
        replace?;
        self.dialog_attribute_at(c, node, position, true)?;
        self.association_attribute_hook(c, node, position)?;
        Ok(())
    }
    pub(super) fn remove_managed_field(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<(), SessionError> {
        self.payloads
            .check_aliases(ConstructionView::Active(self.construction.storage()), c)?;
        let container = self.element_attribute_container(c, node)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        let mut selected = None;
        for (index, attr) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if attr.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attr.name().local_name()), local)?
            {
                selected = Some(index);
                break;
            }
        }
        let Some(index) = selected else {
            return Ok(());
        };
        let style_effect = surgeist_html::KnownHtmlAttribute::from_name(
            element.attributes().as_slice()[index].name(),
        )
        .map(|attribute| {
            surgeist_html::style_attribute_effect(
                element.attributes().as_slice()[index].name().namespace(),
                attribute,
            )
        });
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(c));
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
            if let Some(RetainedPayload::String { location, .. }) =
                self.payloads.values.get_mut(slot)
            {
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
            let Some(RetainedPayload::Attributes(Some(AttributeStorage::Removing {
                rebound, ..
            }))) = self.payloads.values.get_mut(container)
            else {
                return Err(latch_registry_row(c));
            };
            *rebound = slot + 1;
        }
        payload_write::<Option<AttributeStorage>>(c, 2)?;
        payload_write::<usize>(c, 2)?;
        let Some(RetainedPayload::Attributes(storage)) = self.payloads.values.get_mut(container)
        else {
            return Err(latch_registry_row(c));
        };
        let taken = storage.take();
        let Some(AttributeStorage::Removing {
            values,
            allocation,
            shared,
            element,
            ..
        }) = taken
        else {
            *storage = taken;
            return Err(latch_registry_row(c));
        };
        *storage = Some(AttributeStorage::Frozen {
            values,
            allocation,
            shared,
            element,
        });
        if local == "open" {
            self.dialog_open_changed(c, node, true, false)?;
        }
        if style_effect == Some(surgeist_html::HtmlStyleAttributeEffect::OwnerAttributes)
            && self.html_local(c, node, "style")?
        {
            self.record_association(
                c,
                node,
                pending_association::OccurrenceKind::StyleOwnerAttributes,
                None,
                None,
                false,
            )?;
        }
        if base::is_base(
            self.construction
                .node_at(node)
                .ok_or_else(|| latch_registry_row(c))?,
        ) && local == "href"
        {
            self.base_change(c, node, None, true)?;
        }
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
        storage
            .payloads
            .check_aliases(ConstructionView::Active(storage.construction.storage()), c)?;
        let attr = input.as_ref().ok_or(SessionError::Consumed)?;
        c.require_source_range(&attr.source)?;
        check_formatting_text(c, &attr.name, &attr.source)?;
        check_formatting_text(c, &attr.value, &attr.source)?;
        let local = if matches!(family, ManagedFamily::Base)
            && exact_ascii(c, Some(attr.name.value.text()), "href")?
        {
            "href"
        } else if matches!(family, ManagedFamily::Link)
            && exact_ascii(c, Some(attr.name.value.text()), "rel")?
        {
            "rel"
        } else if exact_ascii(c, Some(attr.name.value.text()), "name")? {
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
        let HtmlNodeKind::Element(element) = &storage
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            unreachable!()
        };
        for (index, old) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 2)?;
            if old.name().namespace().uri().is_none()
                && exact_ascii(c, Some(old.name().local_name()), local)?
            {
                position = Some(index);
                break;
            }
        }
        let effect = storage.managed_effect(
            c,
            family,
            node,
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
            payload_write::<Attribute>(c, 1)?;
            let incoming = Attribute::new(QualifiedName::new(Namespace::none(), None, name), value);
            if let Some(position) = position {
                storage.replace_registered_attribute(
                    c,
                    node,
                    [container, position],
                    incoming,
                    [name_slot, value_slot],
                )?;
            } else {
                storage.append_registered_attribute(
                    c,
                    node,
                    container,
                    incoming,
                    [name_slot, value_slot],
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
        let effect = storage.managed_effect(c, family, node, local, Some(old), None)?;
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
